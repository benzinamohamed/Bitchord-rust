mod app;
mod lyrics;
mod models;
mod player;
mod ui;
mod ytmusic;

use app::BitChordApp;
use eframe::egui::Vec2;

fn main() -> Result<(), eframe::Error> {
    // Initialize tokio async runtime for background HTTP & streaming
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize tokio runtime");

    let handle = runtime.handle().clone();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("BitChord — Aesthetic YouTube Music Client")
            .with_inner_size(Vec2::new(1180.0, 780.0))
            .with_min_inner_size(Vec2::new(880.0, 580.0))
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "bitchord",
        options,
        Box::new(move |cc| {
            // Enable egui image loader for remote artwork thumbnails
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(BitChordApp::new(cc, handle)))
        }),
    )
}
