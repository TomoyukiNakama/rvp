// Y'CbCr (planar / semi-planar, 8 or 16 bit) or RGBA → R'G'B', with a
// rounded-rectangle mask. Output is premultiplied alpha (egui's convention).

struct Uniforms {
    // Rows of the Y'CbCr → R'G'B' matrix (xyz used).
    m0: vec4<f32>,
    m1: vec4<f32>,
    m2: vec4<f32>,
    // Destination rect in physical pixels: min.xy, size.xy.
    rect: vec4<f32>,
    // y_offset, y_scale, c_offset, c_scale (range expansion).
    range: vec4<f32>,
    radius: f32,
    // 0 = planar, 1 = semi-planar, 2 = rgba
    plane_layout: u32,
    value_scale: f32,
    output_srgb: u32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var tex0: texture_2d<f32>;
@group(0) @binding(3) var tex1: texture_2d<f32>;
@group(0) @binding(4) var tex2: texture_2d<f32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Triangle strip covering the callback viewport: (0,0) (1,0) (0,1) (1,1).
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2<f32>(f32(i & 1u), f32((i >> 1u) & 1u));
    var out: VsOut;
    out.pos = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

// Coverage of the rounded rect at framebuffer position `p` (1px AA).
fn rounded_coverage(p: vec2<f32>) -> f32 {
    let half = u.rect.zw * 0.5;
    let center = u.rect.xy + half;
    let r = min(u.radius, min(half.x, half.y));
    let q = abs(p - center) - half + vec2<f32>(r);
    let d = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
    return clamp(0.5 - d, 0.0, 1.0);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Sample unconditionally (uniform control flow); pick by layout afterwards.
    let s0 = textureSample(tex0, samp, in.uv);
    let s1 = textureSample(tex1, samp, in.uv);
    let s2 = textureSample(tex2, samp, in.uv);

    var rgb: vec3<f32>;
    if (u.plane_layout == 2u) {
        rgb = s0.rgb;
    } else {
        let y = s0.r * u.value_scale;
        var c: vec2<f32>;
        if (u.plane_layout == 0u) {
            c = vec2<f32>(s1.r, s2.r) * u.value_scale;
        } else {
            c = s1.rg * u.value_scale;
        }
        let yy = (y - u.range.x) * u.range.y;
        let cc = (c - vec2<f32>(u.range.z)) * u.range.w;
        let v = vec3<f32>(yy, cc.x, cc.y);
        rgb = vec3<f32>(dot(u.m0.xyz, v), dot(u.m1.xyz, v), dot(u.m2.xyz, v));
    }
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    if (u.output_srgb == 1u) {
        rgb = srgb_to_linear(rgb);
    }
    let a = rounded_coverage(in.pos.xy);
    return vec4<f32>(rgb * a, a);
}
