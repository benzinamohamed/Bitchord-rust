use crate::models::{HomeShelf, Song};
use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align2, Color32, FontId, Rect, RichText, Rounding, ScrollArea, Sense, Ui,
};

pub struct HomeView;

impl HomeView {
    pub fn show(
        ui: &mut Ui,
        shelves: &[HomeShelf],
        is_loading: bool,
        on_play_song: &mut Option<Song>,
    ) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(20.0);

                // Page Header: Apple Music "Listen Now"
                ui.horizontal(|ui| {
                    ui.add_space(32.0);
                    ui.label(
                        RichText::new("Listen Now")
                            .size(30.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                });

                ui.add_space(20.0);

                if is_loading {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.spinner();
                        ui.add_space(14.0);
                        ui.label(
                            RichText::new("Loading personal shelves from YouTube Music...")
                                .size(14.0)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                    });
                    return;
                }

                // 1. Featured Top Hero Banner Card
                if let Some(first_shelf) = shelves.first() {
                    if let Some(hero_song) = first_shelf.items.first() {
                        ui.horizontal(|ui| {
                            ui.add_space(32.0);
                            let hero_width = (ui.available_width() - 64.0).clamp(320.0, 960.0);
                            let (hero_rect, hero_resp) = ui.allocate_exact_size(vec2(hero_width, 136.0), Sense::click());

                            let hero_bg = if hero_resp.hovered() {
                                BitChordTheme::SURFACE_ACTIVE
                            } else {
                                BitChordTheme::SURFACE
                            };

                            ui.painter().rect_filled(hero_rect, Rounding::same(14.0), hero_bg);
                            ui.painter().rect_stroke(hero_rect, Rounding::same(14.0), BitChordTheme::glass_stroke());

                            // Hero thumbnail
                            let hero_thumb_rect = Rect::from_min_size(
                                hero_rect.min + vec2(14.0, 14.0),
                                vec2(108.0, 108.0),
                            );
                            ui.painter().rect_filled(hero_thumb_rect, Rounding::same(10.0), BitChordTheme::SURFACE_HOVER);

                            if let Some(thumb) = &hero_song.thumbnail_url {
                                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(hero_thumb_rect), |child| {
                                    child.add(egui::Image::new(thumb).rounding(Rounding::same(10.0)));
                                });
                            } else {
                                ui.painter().text(
                                    hero_thumb_rect.center(),
                                    Align2::CENTER_CENTER,
                                    "🎵",
                                    FontId::proportional(36.0),
                                    BitChordTheme::KEY_RED,
                                );
                            }

                            // Hero Info
                            ui.painter().text(
                                hero_rect.min + vec2(136.0, 24.0),
                                Align2::LEFT_TOP,
                                "FEATURED TRACK",
                                FontId::proportional(11.0),
                                BitChordTheme::KEY_RED,
                            );

                            ui.painter().text(
                                hero_rect.min + vec2(136.0, 44.0),
                                Align2::LEFT_TOP,
                                &hero_song.title,
                                FontId::proportional(19.0),
                                BitChordTheme::TEXT_PRIMARY,
                            );

                            ui.painter().text(
                                hero_rect.min + vec2(136.0, 72.0),
                                Align2::LEFT_TOP,
                                &hero_song.artist,
                                FontId::proportional(14.0),
                                BitChordTheme::TEXT_SECONDARY,
                            );

                            // Play Button Pill
                            let play_btn_rect = Rect::from_min_size(
                                hero_rect.min + vec2(136.0, 96.0),
                                vec2(78.0, 26.0),
                            );
                            ui.painter().rect_filled(play_btn_rect, Rounding::same(13.0), BitChordTheme::KEY_RED);
                            ui.painter().text(
                                play_btn_rect.center(),
                                Align2::CENTER_CENTER,
                                "▶  Play",
                                FontId::proportional(12.0),
                                Color32::WHITE,
                            );

                            if hero_resp.clicked() {
                                *on_play_song = Some(hero_song.clone());
                            }
                        });

                        ui.add_space(28.0);
                    }
                }

                // 2. Horizontal Shelf Carousels
                for shelf in shelves {
                    if shelf.items.is_empty() {
                        continue;
                    }

                    ui.horizontal(|ui| {
                        ui.add_space(32.0);
                        ui.label(
                            RichText::new(&shelf.title)
                                .size(21.0)
                                .strong()
                                .color(BitChordTheme::TEXT_PRIMARY),
                        );
                    });

                    ui.add_space(10.0);

                    // Horizontal scrolling shelf row
                    ScrollArea::horizontal()
                        .id_salt(format!("shelf_scroll_{}", shelf.title))
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(32.0);

                                for song in &shelf.items {
                                    let card_size = vec2(150.0, 210.0);
                                    let (rect, resp) = ui.allocate_exact_size(card_size, Sense::click());

                                    let bg = if resp.hovered() {
                                        BitChordTheme::SURFACE_HOVER
                                    } else {
                                        BitChordTheme::SURFACE
                                    };

                                    ui.painter().rect_filled(rect, Rounding::same(10.0), bg);
                                    ui.painter().rect_stroke(
                                        rect,
                                        Rounding::same(10.0),
                                        BitChordTheme::glass_stroke(),
                                    );

                                    // Square Artwork thumbnail
                                    let thumb_rect = Rect::from_min_size(
                                        rect.min + vec2(10.0, 10.0),
                                        vec2(130.0, 130.0),
                                    );
                                    ui.painter().rect_filled(
                                        thumb_rect,
                                        Rounding::same(8.0),
                                        BitChordTheme::SURFACE_ACTIVE,
                                    );

                                    if let Some(thumb) = &song.thumbnail_url {
                                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(thumb_rect), |child| {
                                            child.add(egui::Image::new(thumb).rounding(Rounding::same(8.0)));
                                        });
                                    } else {
                                        ui.painter().text(
                                            thumb_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "🎵",
                                            FontId::proportional(28.0),
                                            BitChordTheme::KEY_RED,
                                        );
                                    }

                                    // Play Button Indicator on Hover
                                    if resp.hovered() {
                                        let play_pos = thumb_rect.max - vec2(20.0, 20.0);
                                        ui.painter().circle_filled(
                                            play_pos,
                                            15.0,
                                            BitChordTheme::KEY_RED,
                                        );
                                        ui.painter().text(
                                            play_pos,
                                            Align2::CENTER_CENTER,
                                            "▶",
                                            FontId::proportional(12.0),
                                            Color32::WHITE,
                                        );
                                    }

                                    // Song Title (Truncated)
                                    let display_title = if song.title.len() > 18 {
                                        format!("{}…", &song.title.chars().take(17).collect::<String>())
                                    } else {
                                        song.title.clone()
                                    };

                                    ui.painter().text(
                                        rect.min + vec2(10.0, 148.0),
                                        Align2::LEFT_TOP,
                                        &display_title,
                                        FontId::proportional(13.0),
                                        BitChordTheme::TEXT_PRIMARY,
                                    );

                                    // Artist Name (Truncated)
                                    let display_artist = if song.artist.len() > 20 {
                                        format!("{}…", &song.artist.chars().take(19).collect::<String>())
                                    } else {
                                        song.artist.clone()
                                    };

                                    ui.painter().text(
                                        rect.min + vec2(10.0, 168.0),
                                        Align2::LEFT_TOP,
                                        &display_artist,
                                        FontId::proportional(12.0),
                                        BitChordTheme::TEXT_SECONDARY,
                                    );

                                    if resp.clicked() {
                                        *on_play_song = Some(song.clone());
                                    }

                                    ui.add_space(14.0);
                                }

                                ui.add_space(32.0);
                            });
                        });

                    ui.add_space(28.0);
                }

                // Add bottom padding so content scrolls completely clear of floating player pill
                ui.add_space(110.0);
            });
    }
}
