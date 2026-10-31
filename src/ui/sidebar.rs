use crate::ui::theme::BitChordTheme;
use egui::{Align, Color32, Layout, Response, RichText, Sense, Ui, Vec2};

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
    pub fn show(ui: &mut Ui, current_tab: &mut NavTab, queue_len: usize) {
        ui.vertical(|ui| {
            ui.add_space(16.0);

            // Brand Header: BitChord
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                // Glowing cyan note indicator
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, BitChordTheme::ACCENT_CYAN);

                ui.add_space(4.0);
                ui.label(
                    RichText::new("BitChord")
                        .size(20.0)
                        .strong()
                        .color(BitChordTheme::TEXT_PRIMARY),
                );
            });

            ui.add_space(24.0);

            // Navigation Items
            Self::nav_item(ui, current_tab, NavTab::Home, "  Listen Now");
            Self::nav_item(ui, current_tab, NavTab::Explore, "  Explore");
            Self::nav_item(ui, current_tab, NavTab::Search, "  Search");
            Self::nav_item(ui, current_tab, NavTab::Library, "  Library");

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);

            Self::nav_item(ui, current_tab, NavTab::Settings, "  Settings");

            // Bottom System Status / Queue badge
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    let badge_text = if queue_len > 0 {
                        format!("Queue: {} tracks", queue_len)
                    } else {
                        "Queue empty".to_string()
                    };
                    ui.label(
                        RichText::new(badge_text)
                            .size(11.0)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new("Arch Linux Native")
                            .size(11.0)
                            .color(BitChordTheme::ACCENT_CYAN),
                    );
                });
            });
        });
    }

    fn nav_item(ui: &mut Ui, current_tab: &mut NavTab, tab: NavTab, label: &str) -> Response {
        let is_selected = *current_tab == tab;
        let mut text = RichText::new(label).size(14.0);

        if is_selected {
            text = text.color(BitChordTheme::ACCENT_CYAN).strong();
        } else {
            text = text.color(BitChordTheme::TEXT_SECONDARY);
        }

        let desired_size = Vec2::new(ui.available_width() - 20.0, 36.0);
        let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click());

        if resp.clicked() {
            *current_tab = tab;
        }

        // Draw selection background
        if is_selected {
            ui.painter().rect_filled(rect, 8.0, BitChordTheme::SURFACE_ACTIVE);
            // Left active pill bar
            let indicator = egui::Rect::from_min_size(
                rect.min + egui::vec2(2.0, 6.0),
                egui::vec2(3.0, rect.height() - 12.0),
            );
            ui.painter().rect_filled(indicator, 2.0, BitChordTheme::ACCENT_CYAN);
        } else if resp.hovered() {
            ui.painter().rect_filled(rect, 8.0, BitChordTheme::SURFACE_HOVER);
        }

        let text_pos = rect.min + egui::vec2(16.0, (rect.height() - 14.0) / 2.0);
        ui.painter().text(
            text_pos,
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(14.0),
            if is_selected {
                BitChordTheme::ACCENT_CYAN
            } else if resp.hovered() {
                Color32::WHITE
            } else {
                BitChordTheme::TEXT_SECONDARY
            },
        );

        resp
    }
}
