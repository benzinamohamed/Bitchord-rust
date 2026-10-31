use crate::models::Song;
use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align2, FontId, RichText, Rounding, ScrollArea, Sense, Ui,
};

pub struct LibraryView;

impl LibraryView {
    pub fn show(
        ui: &mut Ui,
        liked_songs: &[Song],
        history: &[Song],
        on_play: &mut Option<Song>,
    ) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(20.0);

                ui.label(
                    RichText::new("Your Library")
                        .size(32.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Liked tracks and listening history")
                        .size(13.5)
                        .color(BitChordTheme::TEXT_MUTED),
                );

                ui.add_space(24.0);

                // Liked Songs Section
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Liked Songs")
                            .size(20.0)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(format!("({} songs)", liked_songs.len()))
                            .size(14.0)
                            .color(BitChordTheme::ACCENT_PINK),
                    );
                });

                ui.add_space(10.0);

                if liked_songs.is_empty() {
                    ui.label(
                        RichText::new("No liked tracks yet. Click the ♥ icon while playing a song!")
                            .size(13.5)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                } else {
                    for song in liked_songs {
                        Self::render_track_row(ui, song, on_play);
                    }
                }

                ui.add_space(30.0);

                // History Section
                ui.label(
                    RichText::new("Recently Played")
                        .size(20.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.add_space(10.0);

                if history.is_empty() {
                    ui.label(
                        RichText::new("Your playback history will appear here.")
                            .size(13.5)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                } else {
                    for song in history.iter().rev().take(20) {
                        Self::render_track_row(ui, song, on_play);
                    }
                }

                ui.add_space(100.0);
            });
    }

    fn render_track_row(ui: &mut Ui, song: &Song, on_play: &mut Option<Song>) {
        let row_size = vec2(ui.available_width() - 40.0, 48.0);
        let (rect, resp) = ui.allocate_exact_size(row_size, Sense::click());

        let bg = if resp.hovered() {
            BitChordTheme::SURFACE_HOVER
        } else {
            BitChordTheme::SURFACE
        };
        ui.painter().rect_filled(rect, Rounding::same(8.0), bg);

        ui.painter().text(
            rect.min + vec2(16.0, 14.0),
            Align2::CENTER_CENTER,
            "▶",
            FontId::proportional(12.0),
            BitChordTheme::ACCENT_CYAN,
        );

        ui.painter().text(
            rect.min + vec2(36.0, 8.0),
            Align2::LEFT_TOP,
            &song.title,
            FontId::proportional(13.5),
            BitChordTheme::TEXT_PRIMARY,
        );

        ui.painter().text(
            rect.min + vec2(36.0, 26.0),
            Align2::LEFT_TOP,
            &song.artist,
            FontId::proportional(11.5),
            BitChordTheme::TEXT_MUTED,
        );

        if let Some(dur) = &song.duration_text {
            ui.painter().text(
                rect.max - vec2(20.0, 24.0),
                Align2::RIGHT_CENTER,
                dur,
                FontId::proportional(12.0),
                BitChordTheme::TEXT_MUTED,
            );
        }

        if resp.clicked() {
            *on_play = Some(song.clone());
        }

        ui.add_space(2.0);
    }
}
