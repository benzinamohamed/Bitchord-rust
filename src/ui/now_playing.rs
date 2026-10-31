use crate::models::{AudioQualityStats, LyricLine, Song};
use crate::player::AudioPlayer;
use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align, Align2, Color32, FontId, Layout, RichText, Rounding, ScrollArea, Sense, Stroke,
    Ui,
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

        // 1. Draw Ambient Dynamic Mesh Backdrop
        ui.painter().rect_filled(full_rect, Rounding::ZERO, BitChordTheme::BG_DEEP);

        // Ambient radial color glows inspired by Apple Music
        ui.painter().circle_filled(
            full_rect.left_center() + vec2(180.0, -40.0),
            280.0,
            Color32::from_rgba_unmultiplied(250, 45, 72, 22),
        );
        ui.painter().circle_filled(
            full_rect.right_center() + vec2(-180.0, 60.0),
            320.0,
            Color32::from_rgba_unmultiplied(0, 210, 255, 18),
        );

        // 2. Top Bar: Header & Close Button
        ui.horizontal(|ui| {
            ui.add_space(26.0);
            ui.label(
                RichText::new("BitChord • Now Playing")
                    .size(13.5)
                    .strong()
                    .color(BitChordTheme::TEXT_MUTED),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(26.0);
                if ui
                    .add(egui::Button::new(RichText::new("✕").size(18.0).color(Color32::WHITE)).frame(false))
                    .on_hover_text("Close (Esc)")
                    .clicked()
                {
                    *on_close = true;
                }
            });
        });

        ui.add_space(14.0);

        // 3. Split Layout: Left (Artwork + Info), Right (Apple Music Synced Lyrics)
        ui.horizontal(|ui| {
            let left_width = (ui.available_width() * 0.45).clamp(280.0, 460.0);

            // Left Side: Artwork & Nerd Stats
            ui.allocate_ui(vec2(left_width, ui.available_height() - 20.0), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Large Album Artwork Squircle
                    let art_size = (ui.available_width() * 0.78).clamp(200.0, 360.0);
                    let (art_rect, _) = ui.allocate_exact_size(vec2(art_size, art_size), Sense::hover());

                    // Elevation glow
                    ui.painter().rect_filled(
                        art_rect.expand(6.0),
                        Rounding::same(20.0),
                        Color32::from_rgba_unmultiplied(250, 45, 72, 16),
                    );

                    ui.painter().rect_filled(
                        art_rect,
                        Rounding::same(16.0),
                        BitChordTheme::SURFACE_ACTIVE,
                    );
                    ui.painter().rect_stroke(
                        art_rect,
                        Rounding::same(16.0),
                        Stroke::new(1.0_f32, BitChordTheme::GLASS_BORDER),
                    );

                    if let Some(s) = song {
                        if let Some(thumb) = &s.thumbnail_url {
                            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(art_rect), |child| {
                                child.add(egui::Image::new(thumb).rounding(Rounding::same(16.0)));
                            });
                        } else {
                            ui.painter().text(
                                art_rect.center(),
                                Align2::CENTER_CENTER,
                                "🎵",
                                FontId::proportional(72.0),
                                BitChordTheme::KEY_RED,
                            );
                        }
                    }

                    ui.add_space(24.0);

                    // Title & Artist
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

                    ui.add_space(18.0);

                    // Stats for Nerds Badge
                    let nerd_pill = format!(
                        "{} • {} kHz • {} • BitChord Hi-Res",
                        stats.codec,
                        stats.sample_rate_hz / 1000,
                        stats.bit_depth
                    );
                    let (pill_rect, _) = ui.allocate_exact_size(vec2(280.0, 28.0), Sense::hover());
                    ui.painter().rect_filled(
                        pill_rect,
                        Rounding::same(14.0),
                        BitChordTheme::SURFACE_HOVER,
                    );
                    ui.painter().rect_stroke(
                        pill_rect,
                        Rounding::same(14.0),
                        Stroke::new(1.0_f32, BitChordTheme::GLASS_BORDER),
                    );
                    ui.painter().text(
                        pill_rect.center(),
                        Align2::CENTER_CENTER,
                        &nerd_pill,
                        FontId::proportional(11.0),
                        BitChordTheme::KEY_RED,
                    );
                });
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);

            // Right Side: Synchronized Karaoke Lyrics
            let elapsed_ms = player.elapsed_ms();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Lyrics")
                            .size(15.0)
                            .strong()
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new("• Real-Time Synced")
                            .size(11.5)
                            .color(BitChordTheme::KEY_RED),
                    );
                });

                ui.add_space(12.0);

                if lyrics.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(
                            RichText::new("Fetching synchronized lyrics from LRCLIB...")
                                .size(14.5)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                    });
                } else {
                    let active_idx = lyrics
                        .iter()
                        .rposition(|line| elapsed_ms >= line.start_ms)
                        .unwrap_or(0);

                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add_space(40.0);
                            for (idx, line) in lyrics.iter().enumerate() {
                                let is_active = idx == active_idx;

                                let text_color = if is_active {
                                    BitChordTheme::LYRIC_ACTIVE
                                } else {
                                    BitChordTheme::LYRIC_INACTIVE
                                };

                                let font_size = if is_active { 22.0 } else { 16.5 };

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
                                    resp.scroll_to_me(Some(Align::Center));
                                }

                                ui.add_space(18.0);
                            }
                            ui.add_space(140.0);
                        });
                }
            });
        });
    }
}
