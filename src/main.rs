mod app;
mod debug_shot;
mod fonts;
mod player;
mod theme;
mod ui;

use std::path::PathBuf;

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
        ..Default::default()
    };

    eframe::run_native(
        "rvp",
        options,
        Box::new(move |cc| Ok(Box::new(app::RvpApp::new(cc, files)))),
    )
}
