use crate::models::{HomeShelf, Song};
use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align2, Color32, FontId, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui,
};

pub struct HomeView;

impl HomeView {
    pub fn show(ui: &mut Ui, shelves: &[HomeShelf], is_loading: bool, on_play: &mut Option<Song>) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(20.0);

                // Hero Header
                ui.label(
                    RichText::new("Listen Now")
                        .size(32.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Aesthetic YouTube Music streaming powered by Rust")
                        .size(13.5)
                        .color(BitChordTheme::TEXT_MUTED),
                );

                ui.add_space(24.0);

                if is_loading && shelves.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Fetching recommendations from YouTube Music...")
                                .size(14.0)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                    });
                    return;
                }

                if shelves.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(
                            RichText::new("No recommendations loaded. Try searching for a song!")
                                .size(14.0)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                    });
                    return;
                }

                // Render each shelf
                for shelf in shelves {
                    ui.label(
                        RichText::new(&shelf.title)
                            .size(20.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    ui.add_space(12.0);

                    // Horizontal carousel / grid of cards
                    ui.horizontal_wrapped(|ui| {
                        for song in &shelf.items {
                            let card_size = vec2(150.0, 195.0);
                            let (rect, resp) = ui.allocate_exact_size(card_size, Sense::click());

                            // Card Background & Hover
                            let bg = if resp.hovered() {
                                BitChordTheme::SURFACE_HOVER
                            } else {
                                BitChordTheme::SURFACE
                            };
                            ui.painter().rect_filled(rect, Rounding::same(12.0), bg);
                            ui.painter().rect_stroke(
                                rect,
                                Rounding::same(12.0),
                                Stroke::new(1.0, BitChordTheme::GLASS_BORDER),
                            );

                            // Thumbnail Square
                            let thumb_rect = Rect::from_min_size(
                                rect.min + vec2(10.0, 10.0),
                                vec2(card_size.x - 20.0, card_size.x - 20.0),
                            );
                            ui.painter().rect_filled(
                                thumb_rect,
                                Rounding::same(8.0),
                                BitChordTheme::SURFACE_ACTIVE,
                            );

                            // Play Button Indicator on Hover
                            if resp.hovered() {
                                ui.painter().circle_filled(
                                    thumb_rect.center(),
                                    18.0,
                                    BitChordTheme::ACCENT_CYAN,
                                );
                                ui.painter().text(
                                    thumb_rect.center(),
                                    Align2::CENTER_CENTER,
                                    "▶",
                                    FontId::proportional(14.0),
                                    Color32::BLACK,
                                );
                            } else {
                                ui.painter().text(
                                    thumb_rect.center(),
                                    Align2::CENTER_CENTER,
                                    "🎵",
                                    FontId::proportional(32.0),
                                    BitChordTheme::ACCENT_CYAN,
                                );
                            }

                            // Title & Artist Text
                            let title_short = if song.title.len() > 18 {
                                format!("{}...", &song.title[..16])
                            } else {
                                song.title.clone()
                            };
                            ui.painter().text(
                                rect.min + vec2(10.0, card_size.x),
                                Align2::LEFT_TOP,
                                title_short,
                                FontId::proportional(12.5),
                                BitChordTheme::TEXT_PRIMARY,
                            );

                            let artist_short = if song.artist.len() > 20 {
                                format!("{}...", &song.artist[..18])
                            } else {
                                song.artist.clone()
                            };
                            ui.painter().text(
                                rect.min + vec2(10.0, card_size.x + 18.0),
                                Align2::LEFT_TOP,
                                artist_short,
                                FontId::proportional(11.0),
                                BitChordTheme::TEXT_MUTED,
                            );

                            if resp.clicked() {
                                *on_play = Some(song.clone());
                            }

                            ui.add_space(8.0);
                        }
                    });

                    ui.add_space(26.0);
                }

                ui.add_space(100.0); // Bottom padding for player bar
            });
    }
}
