mod app;
mod config;
mod debug_shot;
mod fonts;
mod player;
mod playlist;
mod render;
mod theme;
mod ui;
mod wayland_dnd;

use std::path::PathBuf;
use std::sync::Arc;

use egui_wgpu::wgpu;

/// eframe's default wgpu setup, additionally requesting 16-bit normalized
/// textures when the adapter has them (high-bit-depth video without CPU conversion).
fn wgpu_options() -> egui_wgpu::WgpuConfiguration {
    let mut cfg = egui_wgpu::WgpuConfiguration::default();
    if let egui_wgpu::WgpuSetup::CreateNew(create) = &mut cfg.wgpu_setup {
        let base = create.device_descriptor.clone();
        create.device_descriptor = Arc::new(move |adapter| {
            let mut desc = base(adapter);
            desc.required_features |= adapter.features() & wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
            desc
        });
    }
    cfg
}

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("rvp=info")).init();

    if let Err(e) = player::init() {
        eprintln!("{e:#}");
        std::process::exit(1);
    }

    let files: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rvp")
            .with_app_id("rvp")
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([480.0, 300.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_drag_and_drop(true),
        wgpu_options: wgpu_options(),
        ..Default::default()
    };

    eframe::run_native(
        "rvp",
        options,
        Box::new(move |cc| Ok(Box::new(app::RvpApp::new(cc, files)))),
    )
}
