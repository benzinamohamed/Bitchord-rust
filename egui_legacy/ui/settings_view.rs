use crate::ui::theme::BitChordTheme;
use egui::{vec2, RichText, Rounding, ScrollArea, Stroke, Ui};

pub struct SettingsView;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub audio_quality: String,
    pub discord_rpc: bool,
    pub volume_normalization: bool,
    pub auto_play_similar: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            audio_quality: "High (Opus 160 kbps / 48 kHz)".to_string(),
            discord_rpc: true,
            volume_normalization: true,
            auto_play_similar: true,
        }
    }
}

impl SettingsView {
    pub fn show(ui: &mut Ui, settings: &mut AppSettings) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(20.0);

                ui.label(
                    RichText::new("Settings")
                        .size(32.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Preferences and Arch Linux audio configuration")
                        .size(13.5)
                        .color(BitChordTheme::TEXT_MUTED),
                );

                ui.add_space(24.0);

                // Section 1: Audio Playback
                ui.label(
                    RichText::new("Audio Playback")
                        .size(18.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Audio Quality Stream")
                            .size(14.0)
                            .color(BitChordTheme::TEXT_SECONDARY),
                    );
                    ui.label(
                        RichText::new(&settings.audio_quality)
                            .size(13.0)
                            .color(BitChordTheme::ACCENT_CYAN),
                    );
                    ui.add_space(8.0);

                    ui.checkbox(&mut settings.volume_normalization, "ReplayGain / Volume Normalization");
                    ui.checkbox(&mut settings.auto_play_similar, "Autoplay similar tracks when queue finishes");
                });

                ui.add_space(24.0);

                // Section 2: Integrations
                ui.label(
                    RichText::new("Integrations")
                        .size(18.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.checkbox(&mut settings.discord_rpc, "Discord Rich Presence (Now Playing)");
                    ui.label(
                        RichText::new("MPRIS D-Bus Linux media keys: Enabled natively")
                            .size(12.0)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                });

                ui.add_space(24.0);

                // Section 3: About
                ui.label(
                    RichText::new("About BitChord (Rust Edition)")
                        .size(18.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.add_space(10.0);

                let (info_rect, _) = ui.allocate_exact_size(vec2(ui.available_width() - 40.0, 90.0), egui::Sense::hover());
                ui.painter().rect_filled(info_rect, Rounding::same(12.0), BitChordTheme::SURFACE);
                ui.painter().rect_stroke(info_rect, Rounding::same(12.0), Stroke::new(1.0_f32, BitChordTheme::GLASS_BORDER));

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(info_rect.shrink(14.0)), |child| {
                    child.label(
                        RichText::new("BitChord Rust • Built for Arch Linux")
                            .size(14.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    child.label(
                        RichText::new("Port of BitChord Kotlin app to pure, high-performance safe Rust.")
                            .size(12.5)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                    child.label(
                        RichText::new("Features: YouTube Music API, Time-Synced Lyrics, Apple Music glass UI, Rodio/Symphonia audio engine.")
                            .size(11.5)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                });

                ui.add_space(100.0);
            });
    }
}
