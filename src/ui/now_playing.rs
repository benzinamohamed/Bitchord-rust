use crate::models::{AudioQualityStats, LyricLine, Song};
use crate::player::AudioPlayer;
use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align, Align2, Color32, FontId, Layout, Rect, RichText, Rounding, ScrollArea, Sense,
    Stroke, Ui,
};

pub struct NowPlayingView;

impl NowPlayingView {
    pub fn show(
        ui: &mut Ui,
        player: &AudioPlayer,
        song: Option<&Song>,
        stats: &AudioQualityStats,
        lyrics: &[LyricLine],
        on_seek: &mut Option<u64>,
        on_close: &mut bool,
    ) {
        let full_rect = ui.available_rect_before_wrap();

        // 1. Draw Ambient Gradient Backdrop
        ui.painter().rect_filled(full_rect, Rounding::ZERO, BitChordTheme::BG_DEEP);

        // Ambient radial color glow behind album art
        ui.painter().circle_filled(
            full_rect.left_center() + vec2(150.0, 0.0),
            240.0,
            Color32::from_rgba_unmultiplied(0, 210, 255, 14),
        );
        ui.painter().circle_filled(
            full_rect.center(),
            300.0,
            Color32::from_rgba_unmultiplied(175, 82, 222, 10),
        );

        // 2. Top Bar: Back/Close button & Hi-Res header
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            ui.label(
                RichText::new("Now Playing")
                    .size(13.0)
                    .color(BitChordTheme::TEXT_MUTED),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(24.0);
                if ui
                    .add(egui::Button::new(RichText::new("✕").size(18.0).color(Color32::WHITE)).frame(false))
                    .on_hover_text("Close Now Playing")
                    .clicked()
                {
                    *on_close = true;
                }
            });
        });

        ui.add_space(20.0);

        // 3. Main Split View: Left (Artwork + Info), Right (Synced Lyrics)
        ui.horizontal(|ui| {
            let half_width = ui.available_width() * 0.44;

            // Left Side: Massive Artwork Card & Track Details
            ui.allocate_ui(vec2(half_width, ui.available_height() - 20.0), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Large Album Artwork Squircle
                    let art_size = (ui.available_width() * 0.75).clamp(200.0, 360.0);
                    let (art_rect, _) = ui.allocate_exact_size(vec2(art_size, art_size), Sense::hover());

                    // Ambient glow under album
                    let glow_rect = art_rect.expand(8.0);
                    ui.painter().rect_filled(
                        glow_rect,
                        Rounding::same(24.0),
                        Color32::from_rgba_unmultiplied(0, 210, 255, 20),
                    );

                    // Artwork Body
                    ui.painter().rect_filled(
                        art_rect,
                        Rounding::same(20.0),
                        BitChordTheme::SURFACE_ACTIVE,
                    );
                    ui.painter().rect_stroke(
                        art_rect,
                        Rounding::same(20.0),
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                    );

                    ui.painter().text(
                        art_rect.center(),
                        Align2::CENTER_CENTER,
                        "🎵",
                        FontId::proportional(80.0),
                        BitChordTheme::ACCENT_CYAN,
                    );

                    ui.add_space(28.0);

                    // Song Title & Artist
                    if let Some(s) = song {
                        ui.label(
                            RichText::new(&s.title)
                                .size(22.0)
                                .strong()
                                .color(BitChordTheme::TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&s.artist)
                                .size(16.0)
                                .color(BitChordTheme::TEXT_SECONDARY),
                        );
                    }

                    ui.add_space(20.0);

                    // Stats for Nerds Pill
                    let stats_pill = format!(
                        "{} • {} kHz • {} • BitChord Hi-Res",
                        stats.codec,
                        stats.sample_rate_hz / 1000,
                        stats.bit_depth
                    );
                    let (pill_rect, _) = ui.allocate_exact_size(vec2(280.0, 26.0), Sense::hover());
                    ui.painter().rect_filled(
                        pill_rect,
                        Rounding::same(13.0),
                        BitChordTheme::SURFACE_HOVER,
                    );
                    ui.painter().rect_stroke(
                        pill_rect,
                        Rounding::same(13.0),
                        Stroke::new(1.0, BitChordTheme::GLASS_BORDER),
                    );
                    ui.painter().text(
                        pill_rect.center(),
                        Align2::CENTER_CENTER,
                        &stats_pill,
                        FontId::proportional(11.0),
                        BitChordTheme::ACCENT_CYAN,
                    );
                });
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // Right Side: Apple Music Real-Time Synced Lyrics
            let elapsed_ms = player.elapsed_ms();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Lyrics")
                            .size(14.0)
                            .strong()
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new("• Synced")
                            .size(11.0)
                            .color(BitChordTheme::ACCENT_CYAN),
                    );
                });

                ui.add_space(12.0);

                if lyrics.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(
                            RichText::new("No lyrics available for this track")
                                .size(15.0)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                    });
                } else {
                    // Find active lyric index
                    let active_idx = lyrics
                        .iter()
                        .rposition(|line| elapsed_ms >= line.start_ms)
                        .unwrap_or(0);

                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add_space(30.0);
                            for (idx, line) in lyrics.iter().enumerate() {
                                let is_active = idx == active_idx;

                                let text_color = if is_active {
                                    BitChordTheme::LYRIC_ACTIVE
                                } else {
                                    BitChordTheme::LYRIC_INACTIVE
                                };

                                let font_size = if is_active { 20.0 } else { 16.0 };

                                let mut rich = RichText::new(&line.text)
                                    .size(font_size)
                                    .color(text_color);

                                if is_active {
                                    rich = rich.strong();
                                }

                                let btn = egui::Button::new(rich).frame(false);
                                let resp = ui.add(btn);

                                if resp.clicked() {
                                    *on_seek = Some(line.start_ms);
                                }

                                if is_active {
                                    // Smoothly scroll active line into middle
                                    resp.scroll_to_me(Some(Align::Center));
                                }

                                ui.add_space(16.0);
                            }
                            ui.add_space(120.0);
                        });
                }
            });
        });
    }
}
