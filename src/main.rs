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
            .with_title("BitChord — Apple Music Client for Arch Linux")
            .with_inner_size(Vec2::new(1200.0, 800.0))
            .with_min_inner_size(Vec2::new(900.0, 600.0))
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "bitchord",
        options,
        Box::new(move |cc| {
            // Enable egui image loader for remote artwork thumbnails
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // Install Inter font for Apple Music typography
            let font_paths = [
                "/home/hamoda/.local/share/fonts/Inter/extras/ttf/Inter-Regular.ttf",
                "/usr/share/fonts/inter/Inter-Regular.ttf",
            ];
            for path in font_paths {
                if let Ok(font_bytes) = std::fs::read(path) {
                    let mut fonts = egui::FontDefinitions::default();
                    fonts.font_data.insert(
                        "Inter".to_owned(),
                        egui::FontData::from_owned(font_bytes),
                    );
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        family.insert(0, "Inter".to_owned());
                    }
                    cc.egui_ctx.set_fonts(fonts);
                    break;
                }
            }

            Ok(Box::new(BitChordApp::new(cc, handle)))
        }),
    )
}
