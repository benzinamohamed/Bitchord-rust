use crate::models::{SearchResult, Song};
use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align2, Color32, FontId, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, TextEdit,
    Ui,
};

pub struct SearchView;

impl SearchView {
    pub fn show(
        ui: &mut Ui,
        search_query: &mut String,
        results: &SearchResult,
        is_searching: bool,
        on_trigger_search: &mut bool,
        on_play: &mut Option<Song>,
    ) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(20.0);

                // Search Bar Input Field
                ui.horizontal(|ui| {
                    let search_bar_width = (ui.available_width() - 80.0).clamp(240.0, 600.0);
                    let (search_rect, _) =
                        ui.allocate_exact_size(vec2(search_bar_width, 42.0), Sense::hover());

                    ui.painter().rect_filled(
                        search_rect,
                        Rounding::same(21.0),
                        BitChordTheme::SURFACE_HOVER,
                    );
                    ui.painter().rect_stroke(
                        search_rect,
                        Rounding::same(21.0),
                        BitChordTheme::glass_stroke(),
                    );

                    // Magnifying glass icon inside pill
                    ui.painter().text(
                        search_rect.min + vec2(16.0, 12.0),
                        Align2::LEFT_TOP,
                        "🔍",
                        FontId::proportional(15.0),
                        BitChordTheme::TEXT_MUTED,
                    );

                    let edit_rect = Rect::from_min_size(
                        search_rect.min + vec2(42.0, 6.0),
                        vec2(search_bar_width - 60.0, 30.0),
                    );
                    let mut edit_lost_focus = false;
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(edit_rect), |child| {
                        let edit = TextEdit::singleline(search_query)
                            .hint_text("Search songs, artists, albums...")
                            .text_color(BitChordTheme::TEXT_PRIMARY)
                            .frame(false);

                        let edit_resp = child.add(edit);
                        if edit_resp.lost_focus() {
                            edit_lost_focus = true;
                        }
                    });
                    if edit_lost_focus && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        *on_trigger_search = true;
                    }

                    // Search Button
                    let search_btn = egui::Button::new(
                        RichText::new("Search")
                            .size(13.0)
                            .strong()
                            .color(Color32::BLACK),
                    )
                    .fill(BitChordTheme::ACCENT_CYAN)
                    .rounding(Rounding::same(18.0));

                    if ui.add_sized(vec2(76.0, 36.0), search_btn).clicked() {
                        *on_trigger_search = true;
                    }
                });

                ui.add_space(24.0);

                if is_searching {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Searching YouTube Music...")
                                .size(14.0)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                    });
                    return;
                }

                // If no search has happened yet, display suggestions
                if results.songs.is_empty() && results.query.is_empty() {
                    ui.label(
                        RichText::new("Popular Searches")
                            .size(18.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    ui.add_space(12.0);

                    let suggestions = [
                        "The Weeknd",
                        "Taylor Swift",
                        "Daft Punk",
                        "Lofi Hip Hop",
                        "Hans Zimmer",
                        "Queen",
                        "Billie Eilish",
                        "Coldplay",
                    ];

                    ui.horizontal_wrapped(|ui| {
                        for sugg in suggestions {
                            let btn = egui::Button::new(
                                RichText::new(sugg)
                                    .size(13.0)
                                    .color(BitChordTheme::TEXT_SECONDARY),
                            )
                            .fill(BitChordTheme::SURFACE_HOVER)
                            .rounding(Rounding::same(16.0));

                            if ui.add(btn).clicked() {
                                *search_query = sugg.to_string();
                                *on_trigger_search = true;
                            }
                            ui.add_space(8.0);
                        }
                    });
                    return;
                }

                // Top Result Card (if available)
                if let Some(top) = &results.top_result {
                    ui.label(
                        RichText::new("Top Result")
                            .size(18.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    ui.add_space(10.0);

                    let (card_rect, card_resp) =
                        ui.allocate_exact_size(vec2(ui.available_width() - 40.0, 80.0), Sense::click());
                    let bg = if card_resp.hovered() {
                        BitChordTheme::SURFACE_ACTIVE
                    } else {
                        BitChordTheme::SURFACE
                    };
                    ui.painter().rect_filled(card_rect, Rounding::same(14.0), bg);
                    ui.painter().rect_stroke(
                        card_rect,
                        Rounding::same(14.0),
                        Stroke::new(1.0_f32, BitChordTheme::GLASS_BORDER),
                    );

                    // Album Art Box
                    let thumb_rect = Rect::from_min_size(
                        card_rect.min + vec2(14.0, 14.0),
                        vec2(52.0, 52.0),
                    );
                    ui.painter().rect_filled(
                        thumb_rect,
                        Rounding::same(8.0),
                        BitChordTheme::SURFACE_ACTIVE,
                    );
                    ui.painter().text(
                        thumb_rect.center(),
                        Align2::CENTER_CENTER,
                        "▶",
                        FontId::proportional(22.0),
                        BitChordTheme::ACCENT_CYAN,
                    );

                    // Title & Artist
                    ui.painter().text(
                        card_rect.min + vec2(80.0, 20.0),
                        Align2::LEFT_TOP,
                        &top.title,
                        FontId::proportional(16.0),
                        BitChordTheme::TEXT_PRIMARY,
                    );
                    ui.painter().text(
                        card_rect.min + vec2(80.0, 44.0),
                        Align2::LEFT_TOP,
                        format!("{} • Song", top.artist),
                        FontId::proportional(13.0),
                        BitChordTheme::TEXT_MUTED,
                    );

                    if card_resp.clicked() {
                        *on_play = Some(top.clone());
                    }

                    ui.add_space(20.0);
                }

                // Songs Results List
                if !results.songs.is_empty() {
                    ui.label(
                        RichText::new("Songs")
                            .size(18.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    ui.add_space(10.0);

                    for (idx, song) in results.songs.iter().enumerate() {
                        let row_size = vec2(ui.available_width() - 40.0, 52.0);
                        let (row_rect, row_resp) = ui.allocate_exact_size(row_size, Sense::click());

                        let bg = if row_resp.hovered() {
                            BitChordTheme::SURFACE_HOVER
                        } else if idx % 2 == 0 {
                            BitChordTheme::SURFACE
                        } else {
                            Color32::TRANSPARENT
                        };
                        ui.painter().rect_filled(row_rect, Rounding::same(8.0), bg);

                        // Track Number / Play icon
                        let play_icon = if row_resp.hovered() { "▶" } else { "•" };
                        ui.painter().text(
                            row_rect.min + vec2(16.0, 16.0),
                            Align2::CENTER_CENTER,
                            play_icon,
                            FontId::proportional(12.0),
                            BitChordTheme::ACCENT_CYAN,
                        );

                        // Title
                        let title_text = if song.title.len() > 40 {
                            format!("{}...", &song.title[..38])
                        } else {
                            song.title.clone()
                        };
                        ui.painter().text(
                            row_rect.min + vec2(36.0, 10.0),
                            Align2::LEFT_TOP,
                            title_text,
                            FontId::proportional(13.5),
                            BitChordTheme::TEXT_PRIMARY,
                        );

                        // Artist
                        ui.painter().text(
                            row_rect.min + vec2(36.0, 28.0),
                            Align2::LEFT_TOP,
                            &song.artist,
                            FontId::proportional(11.5),
                            BitChordTheme::TEXT_MUTED,
                        );

                        // Duration text on the right
                        if let Some(dur) = &song.duration_text {
                            ui.painter().text(
                                row_rect.max - vec2(20.0, 26.0),
                                Align2::RIGHT_CENTER,
                                dur,
                                FontId::proportional(12.0),
                                BitChordTheme::TEXT_MUTED,
                            );
                        }

                        if row_resp.clicked() {
                            *on_play = Some(song.clone());
                        }

                        ui.add_space(2.0);
                    }
                }

                ui.add_space(100.0);
            });
    }
}
