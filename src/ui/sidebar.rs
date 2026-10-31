use crate::ui::theme::BitChordTheme;
use egui::{Align, Color32, Layout, Response, RichText, Rounding, Sense, TextEdit, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Home,
    Explore,
    Search,
    Library,
    Settings,
}

pub struct Sidebar;

impl Sidebar {
    pub fn show(
        ui: &mut Ui,
        current_tab: &mut NavTab,
        search_query: &mut String,
        on_trigger_search: &mut bool,
        queue_len: usize,
    ) {
        ui.vertical(|ui| {
            ui.add_space(20.0);

            // Brand Header: BitChord LogoMark (Apple Music red / BitChord Cyan)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
                ui.painter().rect_filled(rect, Rounding::same(6.0), BitChordTheme::KEY_RED);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "♪",
                    egui::FontId::proportional(15.0),
                    Color32::WHITE,
                );

                ui.add_space(8.0);
                ui.label(
                    RichText::new("BitChord")
                        .size(18.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
            });

            ui.add_space(18.0);

            // Embedded Apple Music Pill Search Input
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                let search_width = ui.available_width() - 28.0;
                let (box_rect, _) = ui.allocate_exact_size(Vec2::new(search_width, 32.0), Sense::hover());

                ui.painter().rect_filled(
                    box_rect,
                    Rounding::same(8.0),
                    BitChordTheme::SURFACE_HOVER,
                );
                ui.painter().rect_stroke(
                    box_rect,
                    Rounding::same(8.0),
                    BitChordTheme::glass_stroke(),
                );

                ui.painter().text(
                    box_rect.min + egui::vec2(10.0, 7.0),
                    egui::Align2::LEFT_TOP,
                    "🔍",
                    egui::FontId::proportional(13.0),
                    BitChordTheme::TEXT_MUTED,
                );

                let edit_rect = egui::Rect::from_min_size(
                    box_rect.min + egui::vec2(30.0, 4.0),
                    Vec2::new(search_width - 36.0, 24.0),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(edit_rect), |child| {
                    let edit = TextEdit::singleline(search_query)
                        .hint_text("Search...")
                        .text_color(BitChordTheme::TEXT_PRIMARY)
                        .frame(false);

                    let edit_resp = child.add(edit);
                    let enter_pressed = child.input(|i| i.key_pressed(egui::Key::Enter));
                    if edit_resp.lost_focus() && enter_pressed {
                        *current_tab = NavTab::Search;
                        *on_trigger_search = true;
                    }
                });
            });

            ui.add_space(18.0);

            // Primary Navigation Items
            Self::nav_item(ui, current_tab, NavTab::Search, "🔍", "Search");
            Self::nav_item(ui, current_tab, NavTab::Home, "🏠", "Listen Now");
            Self::nav_item(ui, current_tab, NavTab::Explore, "🧭", "Browse");

            ui.add_space(16.0);

            // LIBRARY Section Header
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new("LIBRARY")
                        .size(11.0)
                        .strong()
                        .color(BitChordTheme::TEXT_MUTED),
                );
            });
            ui.add_space(6.0);

            Self::nav_item(ui, current_tab, NavTab::Library, "♥", "Liked Songs");
            Self::nav_item(ui, current_tab, NavTab::Library, "⏱", "Recently Added");

            ui.add_space(16.0);

            // PLAYLISTS Section Header
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new("PLAYLISTS")
                        .size(11.0)
                        .strong()
                        .color(BitChordTheme::TEXT_MUTED),
                );
            });
            ui.add_space(6.0);
            Self::nav_item(ui, current_tab, NavTab::Home, "🔥", "Top Charts");
            Self::nav_item(ui, current_tab, NavTab::Explore, "✨", "Community Mixes");

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(10.0);

            Self::nav_item(ui, current_tab, NavTab::Settings, "⚙", "Settings");

            // Bottom Queue Status indicator
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    let badge_text = if queue_len > 0 {
                        format!("Queue: {} tracks", queue_len)
                    } else {
                        "Queue empty".to_string()
                    };
                    ui.label(
                        RichText::new(badge_text)
                            .size(11.5)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                });
            });
        });
    }

    fn nav_item(
        ui: &mut Ui,
        current_tab: &mut NavTab,
        target_tab: NavTab,
        icon: &str,
        label: &str,
    ) -> Response {
        let is_selected = *current_tab == target_tab;
        let item_width = ui.available_width() - 20.0;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_width, 36.0), Sense::click());

        // Apple Music selection highlight
        let bg = if is_selected {
            BitChordTheme::SURFACE_ACTIVE
        } else if resp.hovered() {
            BitChordTheme::SURFACE_HOVER
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, Rounding::same(8.0), bg);
        }

        // Active left pill indicator
        if is_selected {
            let ind_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(2.0, 9.0),
                Vec2::new(3.0, 18.0),
            );
            ui.painter().rect_filled(ind_rect, Rounding::same(2.0), BitChordTheme::KEY_RED);
        }

        let text_color = if is_selected {
            Color32::WHITE
        } else if resp.hovered() {
            BitChordTheme::TEXT_PRIMARY
        } else {
            BitChordTheme::TEXT_SECONDARY
        };

        // Icon
        ui.painter().text(
            rect.min + egui::vec2(16.0, 9.0),
            egui::Align2::LEFT_TOP,
            icon,
            egui::FontId::proportional(15.0),
            if is_selected { BitChordTheme::KEY_RED } else { BitChordTheme::TEXT_MUTED },
        );

        // Label
        ui.painter().text(
            rect.min + egui::vec2(44.0, 10.0),
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(13.5),
            text_color,
        );

        if resp.clicked() {
            *current_tab = target_tab;
        }

        resp
    }
}
