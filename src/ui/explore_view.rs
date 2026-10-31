use crate::ui::theme::BitChordTheme;
use egui::{
    vec2, Align2, Color32, FontId, RichText, Rounding, ScrollArea, Sense, Stroke, Ui,
};

pub struct ExploreView;

struct GenreCategory {
    name: &'static str,
    color: Color32,
    icon: &'static str,
}

impl ExploreView {
    pub fn show(ui: &mut Ui, on_select_genre: &mut Option<String>) {
        let categories = [
            GenreCategory {
                name: "Pop",
                color: Color32::from_rgb(250, 45, 72),
                icon: "🎤",
            },
            GenreCategory {
                name: "Hip-Hop",
                color: Color32::from_rgb(255, 149, 0),
                icon: "🎧",
            },
            GenreCategory {
                name: "Electronic / Dance",
                color: Color32::from_rgb(0, 210, 255),
                icon: "⚡",
            },
            GenreCategory {
                name: "Rock & Alternative",
                color: Color32::from_rgb(175, 82, 222),
                icon: "🎸",
            },
            GenreCategory {
                name: "R&B / Soul",
                color: Color32::from_rgb(88, 86, 214),
                icon: "🎷",
            },
            GenreCategory {
                name: "Lo-Fi & Chill",
                color: Color32::from_rgb(52, 199, 89),
                icon: "☕",
            },
            GenreCategory {
                name: "Indie",
                color: Color32::from_rgb(255, 45, 85),
                icon: "🌙",
            },
            GenreCategory {
                name: "Gaming & Synthwave",
                color: Color32::from_rgb(90, 200, 250),
                icon: "🎮",
            },
            GenreCategory {
                name: "Classical",
                color: Color32::from_rgb(162, 132, 94),
                icon: "🎻",
            },
            GenreCategory {
                name: "Ambient & Focus",
                color: Color32::from_rgb(142, 142, 147),
                icon: "🌊",
            },
        ];

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(20.0);

                ui.label(
                    RichText::new("Explore")
                        .size(32.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Browse charts, moods, and musical genres")
                        .size(13.5)
                        .color(BitChordTheme::TEXT_MUTED),
                );

                ui.add_space(24.0);

                ui.label(
                    RichText::new("Moods & Genres")
                        .size(20.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
                ui.add_space(14.0);

                // Grid of genre tiles
                ui.horizontal_wrapped(|ui| {
                    for cat in categories {
                        let tile_size = vec2(170.0, 95.0);
                        let (rect, resp) = ui.allocate_exact_size(tile_size, Sense::click());

                        let bg = if resp.hovered() {
                            cat.color
                        } else {
                            BitChordTheme::SURFACE
                        };

                        ui.painter().rect_filled(rect, Rounding::same(14.0), bg);
                        ui.painter().rect_stroke(
                            rect,
                            Rounding::same(14.0),
                            Stroke::new(1.0_f32, BitChordTheme::GLASS_BORDER),
                        );

                        // Icon in top right
                        ui.painter().text(
                            rect.max - vec2(16.0, 24.0),
                            Align2::RIGHT_CENTER,
                            cat.icon,
                            FontId::proportional(26.0),
                            Color32::WHITE,
                        );

                        // Category Name in bottom left
                        let text_color = if resp.hovered() {
                            Color32::BLACK
                        } else {
                            BitChordTheme::TEXT_PRIMARY
                        };
                        ui.painter().text(
                            rect.min + vec2(14.0, 56.0),
                            Align2::LEFT_TOP,
                            cat.name,
                            FontId::proportional(14.0),
                            text_color,
                        );

                        if resp.clicked() {
                            *on_select_genre = Some(format!("Top {}", cat.name));
                        }

                        ui.add_space(10.0);
                    }
                });

                ui.add_space(100.0);
            });
    }
}
