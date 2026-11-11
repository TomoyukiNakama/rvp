//! GPU video rendering: uploads YUV planes as textures and converts to RGB in
//! a shader, drawn inside egui's render pass via a paint callback.

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

use crate::player::{Matrix, PlaneLayout, VideoFrame};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    m0: [f32; 4],
    m1: [f32; 4],
    m2: [f32; 4],
    rect: [f32; 4],
    range: [f32; 4],
    radius: f32,
    plane_layout: u32,
    value_scale: f32,
    output_srgb: u32,
}

/// Identifies a texture set that can be reused across frames.
#[derive(Clone, PartialEq, Eq)]
struct TexKey {
    layout: PlaneLayout,
    high_depth: bool,
    sizes: Vec<(u32, u32)>,
}

struct TextureSet {
    key: TexKey,
    textures: Vec<wgpu::Texture>,
    bind_group: wgpu::BindGroup,
}

/// Long-lived GPU state, stored in egui-wgpu's `CallbackResources`.
pub struct VideoRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    dummy: wgpu::TextureView,
    output_srgb: bool,
    set: Option<TextureSet>,
    uploaded: u64,
}

impl VideoRenderer {
    /// Registers the renderer with egui-wgpu. Call once at startup.
    pub fn install(rs: &egui_wgpu::RenderState) {
        let r = Self::new(&rs.device, &rs.queue, rs.target_format);
        rs.renderer.write().callback_resources.insert(r);
    }

    fn new(device: &wgpu::Device, queue: &wgpu::Queue, target: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rvp video shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("video.wgsl").into()),
        });
        let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rvp video bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex_entry(2),
                tex_entry(3),
                tex_entry(4),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rvp video pl"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rvp video pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rvp video sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rvp video uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dummy_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rvp dummy"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            dummy_tex.as_image_copy(),
            &[128],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(1), rows_per_image: None },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        Self {
            pipeline,
            layout,
            sampler,
            uniforms,
            dummy: dummy_tex.create_view(&Default::default()),
            output_srgb: target.is_srgb(),
            set: None,
            uploaded: 0,
        }
    }

    fn texture_format(components: u32, high_depth: bool) -> wgpu::TextureFormat {
        use wgpu::TextureFormat as F;
        match (components, high_depth) {
            (1, false) => F::R8Unorm,
            (2, false) => F::Rg8Unorm,
            (1, true) => F::R16Unorm,
            (2, true) => F::Rg16Unorm,
            _ => F::Rgba8Unorm,
        }
    }

    fn ensure_textures(&mut self, device: &wgpu::Device, f: &VideoFrame) {
        let key = TexKey {
            layout: f.layout,
            high_depth: f.high_depth,
            sizes: f.planes.iter().map(|p| (p.width, p.height)).collect(),
        };
        if self.set.as_ref().is_some_and(|s| s.key == key) {
            return;
        }
        let textures: Vec<wgpu::Texture> = f
            .planes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("rvp plane {i}")),
                    size: wgpu::Extent3d { width: p.width, height: p.height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: Self::texture_format(p.components, f.high_depth),
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                })
            })
            .collect();
        let views: Vec<wgpu::TextureView> = textures.iter().map(|t| t.create_view(&Default::default())).collect();
        let view = |i: usize| views.get(i).unwrap_or(&self.dummy);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rvp video bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniforms.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(view(0)) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(view(1)) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(view(2)) },
            ],
        });
        log::debug!("video textures: {:?} high_depth={} {:?}", key.layout, key.high_depth, key.sizes);
        self.set = Some(TextureSet { key, textures, bind_group });
    }

    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, f: &VideoFrame) {
        if self.uploaded == f.id {
            return;
        }
        self.ensure_textures(device, f);
        let Some(set) = &self.set else { return };
        let bps = f.bytes_per_sample();
        for (i, (p, tex)) in f.planes.iter().zip(&set.textures).enumerate() {
            let (data, stride) = f.plane_data(i);
            let row_bytes = (p.width * p.components * bps) as usize;
            let needed = stride * (p.height as usize - 1) + row_bytes;
            if data.len() < needed || stride < row_bytes {
                log::error!("plane {i}: {} bytes, stride {stride}, need {needed}", data.len());
                return;
            }
            queue.write_texture(
                tex.as_image_copy(),
                &data[..needed],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride as u32),
                    rows_per_image: Some(p.height),
                },
                wgpu::Extent3d { width: p.width, height: p.height, depth_or_array_layers: 1 },
            );
        }
        self.uploaded = f.id;
    }

    fn write_uniforms(&self, queue: &wgpu::Queue, f: &VideoFrame, rect_px: [f32; 4], radius_px: f32) {
        let (m, range) = color_params(f.matrix, f.bits, f.full_range);
        let u = Uniforms {
            m0: m[0],
            m1: m[1],
            m2: m[2],
            rect: rect_px,
            range,
            radius: radius_px,
            plane_layout: match f.layout {
                PlaneLayout::Planar => 0,
                PlaneLayout::SemiPlanar => 1,
                PlaneLayout::Rgba => 2,
            },
            value_scale: f.value_scale,
            output_srgb: u32::from(self.output_srgb),
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&u));
    }
}

/// Matrix rows and range expansion (`y_off, y_scale, c_off, c_scale`) for
/// `bits`-deep codes normalized to [0, 1]. Mirrors the math in `video.wgsl`.
fn color_params(matrix: Matrix, bits: u32, full_range: bool) -> ([[f32; 4]; 3], [f32; 4]) {
    let m = matrix.rows();
    let row = |r: [f32; 3]| [r[0], r[1], r[2], 0.0];
    let n = bits.max(8);
    let max = ((1u32 << n) - 1) as f32;
    let unit = (1u32 << (n - 8)) as f32; // 1 for 8-bit, 4 for 10-bit, ...
    let c_off = (1u32 << (n - 1)) as f32 / max;
    let range = if full_range {
        [0.0, 1.0, c_off, 1.0]
    } else {
        [16.0 * unit / max, max / (219.0 * unit), c_off, max / (224.0 * unit)]
    };
    ([row(m[0]), row(m[1]), row(m[2])], range)
}

/// Per-frame paint request for the video quad.
pub struct VideoPaint {
    pub frame: Arc<VideoFrame>,
    pub rect: egui::Rect,
    /// Corner radius in points.
    pub radius: f32,
}

impl CallbackTrait for VideoPaint {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(r) = resources.get_mut::<VideoRenderer>() else {
            return Vec::new();
        };
        let ppp = screen.pixels_per_point;
        r.upload(device, queue, &self.frame);
        let rect = [
            self.rect.min.x * ppp,
            self.rect.min.y * ppp,
            self.rect.width() * ppp,
            self.rect.height() * ppp,
        ];
        r.write_uniforms(queue, &self.frame, rect, self.radius * ppp);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let Some(r) = resources.get::<VideoRenderer>() else { return };
        let Some(set) = &r.set else { return };
        if r.uploaded != self.frame.id {
            return; // upload failed; draw nothing rather than a stale/garbage frame
        }
        pass.set_pipeline(&r.pipeline);
        pass.set_bind_group(0, &set.bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_wgpu::wgpu::naga;

    #[test]
    fn shader_parses_and_validates() {
        let module = naga::front::wgsl::parse_str(include_str!("video.wgsl")).expect("WGSL parse error");
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty())
            .validate(&module)
            .expect("WGSL validation error");
    }

    #[test]
    fn uniform_layout_matches_wgsl() {
        // 5 × vec4 + 4 × scalar, no implicit padding.
        assert_eq!(std::mem::size_of::<Uniforms>(), 96);
    }

    /// Same formula as `fs_main`, on integer codes.
    fn to_rgb(matrix: Matrix, bits: u32, full: bool, y: u32, cb: u32, cr: u32) -> [f32; 3] {
        let (m, r) = color_params(matrix, bits, full);
        let max = ((1u32 << bits) - 1) as f32;
        let (y, cb, cr) = (y as f32 / max, cb as f32 / max, cr as f32 / max);
        let v = [(y - r[0]) * r[1], (cb - r[2]) * r[3], (cr - r[2]) * r[3]];
        m.map(|row| (row[0] * v[0] + row[1] * v[1] + row[2] * v[2]).clamp(0.0, 1.0))
    }

    fn approx(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 0.006, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn limited_range_black_white_and_primaries() {
        for m in [Matrix::Bt601, Matrix::Bt709, Matrix::Bt2020] {
            approx(to_rgb(m, 8, false, 16, 128, 128), [0.0; 3]);
            approx(to_rgb(m, 8, false, 235, 128, 128), [1.0; 3]);
            approx(to_rgb(m, 10, false, 64, 512, 512), [0.0; 3]);
            approx(to_rgb(m, 10, false, 940, 512, 512), [1.0; 3]);
        }
        // BT.709 100% red is Y'CbCr (63, 102, 240) in 8-bit limited range.
        approx(to_rgb(Matrix::Bt709, 8, false, 63, 102, 240), [1.0, 0.0, 0.0]);
        // BT.601 100% green: (145, 54, 34).
        approx(to_rgb(Matrix::Bt601, 8, false, 145, 54, 34), [0.0, 1.0, 0.0]);
        // Full range (JPEG): mid grey stays mid grey.
        approx(to_rgb(Matrix::Bt601, 8, true, 128, 128, 128), [128.0 / 255.0; 3]);
    }
}
