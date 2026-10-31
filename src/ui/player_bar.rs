use crate::models::{AudioQualityStats, PlaybackStatus, RepeatMode, Song};
use crate::player::AudioPlayer;
use crate::ui::theme::BitChordTheme;
use egui::{
    pos2, vec2, Align2, Color32, FontId, Rect, RichText, Rounding, Sense, Stroke, Ui,
};

pub struct PlayerBarAction {
    pub toggle_play: bool,
    pub next: bool,
    pub previous: bool,
    pub toggle_shuffle: bool,
    pub cycle_repeat: bool,
    pub toggle_lyrics: bool,
    pub toggle_queue: bool,
    pub toggle_fullscreen: bool,
    pub toggle_like: bool,
    pub seek_to_ms: Option<u64>,
}

pub struct PlayerBar;

impl PlayerBar {
    pub fn show(
        ui: &mut Ui,
        player: &AudioPlayer,
        current_song: Option<&Song>,
        stats: &AudioQualityStats,
        repeat_mode: RepeatMode,
        shuffle: bool,
        lyrics_open: bool,
        queue_open: bool,
        is_liked: bool,
    ) -> PlayerBarAction {
        let mut action = PlayerBarAction {
            toggle_play: false,
            next: false,
            previous: false,
            toggle_shuffle: false,
            cycle_repeat: false,
            toggle_lyrics: false,
            toggle_queue: false,
            toggle_fullscreen: false,
            toggle_like: false,
            seek_to_ms: None,
        };

        let bar_height = 56.0;
        let width = (ui.available_width() - 40.0).clamp(320.0, 780.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), bar_height + 14.0), Sense::hover());
        let pill_rect = Rect::from_center_size(rect.center(), vec2(width, bar_height));

        // Draw Apple Music Floating Glass Pill with border and subtle glow
        ui.painter().rect_filled(pill_rect, Rounding::same(28.0), BitChordTheme::GLASS_PILL);
        ui.painter().rect_stroke(pill_rect, Rounding::same(28.0), BitChordTheme::glass_stroke());

        // Left 180px: Transport Controls
        let left_rect = Rect::from_min_size(
            pill_rect.min + vec2(12.0, 4.0),
            vec2(160.0, bar_height - 8.0),
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |left_ui| {
            Self::render_transports(left_ui, player, repeat_mode, shuffle, &mut action);
        });

        // Center LCD: Artwork + Title/Artist + Scrubber
        let center_width = (pill_rect.width() - 360.0).max(180.0);
        let center_rect = Rect::from_min_size(
            pill_rect.min + vec2(174.0, 4.0),
            vec2(center_width, bar_height - 8.0),
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(center_rect), |center_ui| {
            Self::render_lcd(center_ui, player, current_song, &mut action);
        });

        // Right 180px: Actions & Volume
        let right_rect = Rect::from_min_size(
            pos2(pill_rect.max.x - 180.0, pill_rect.min.y + 4.0),
            vec2(170.0, bar_height - 8.0),
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |right_ui| {
            Self::render_right_actions(
                right_ui,
                player,
                stats,
                lyrics_open,
                queue_open,
                is_liked,
                &mut action,
            );
        });

        action
    }

    fn render_transports(
        ui: &mut Ui,
        player: &AudioPlayer,
        repeat_mode: RepeatMode,
        shuffle: bool,
        action: &mut PlayerBarAction,
    ) {
        let status = player.status();
        let is_playing = status == PlaybackStatus::Playing;
        let is_buffering = status == PlaybackStatus::Buffering;

        ui.horizontal_centered(|ui| {
            // Shuffle
            let shuffle_color = if shuffle {
                BitChordTheme::KEY_CYAN
            } else {
                BitChordTheme::TEXT_MUTED
            };
            if ui
                .add(egui::Button::new(RichText::new("🔀").size(14.0).color(shuffle_color)).frame(false))
                .clicked()
            {
                action.toggle_shuffle = true;
            }

            ui.add_space(4.0);

            // Previous
            if ui
                .add(egui::Button::new(RichText::new("⏮").size(16.0).color(BitChordTheme::TEXT_PRIMARY)).frame(false))
                .clicked()
            {
                action.previous = true;
            }

            ui.add_space(4.0);

            // Play / Pause Circle
            let (play_rect, play_resp) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
            let play_bg = if is_playing {
                BitChordTheme::KEY_CYAN
            } else {
                BitChordTheme::SURFACE_ACTIVE
            };
            ui.painter().circle_filled(play_rect.center(), 14.0, play_bg);

            let icon = if is_buffering {
                "⏳"
            } else if is_playing {
                "⏸"
            } else {
                "▶"
            };
            let icon_color = if is_playing {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            ui.painter().text(
                play_rect.center(),
                Align2::CENTER_CENTER,
                icon,
                FontId::proportional(13.0),
                icon_color,
            );

            if play_resp.clicked() {
                action.toggle_play = true;
            }

            ui.add_space(4.0);

            // Next
            if ui
                .add(egui::Button::new(RichText::new("⏭").size(16.0).color(BitChordTheme::TEXT_PRIMARY)).frame(false))
                .clicked()
            {
                action.next = true;
            }

            ui.add_space(4.0);

            // Repeat
            let repeat_color = match repeat_mode {
                RepeatMode::Off => BitChordTheme::TEXT_MUTED,
                RepeatMode::All => BitChordTheme::KEY_CYAN,
                RepeatMode::One => BitChordTheme::KEY_RED,
            };
            let repeat_icon = if repeat_mode == RepeatMode::One { "🔂" } else { "🔁" };
            if ui
                .add(egui::Button::new(RichText::new(repeat_icon).size(14.0).color(repeat_color)).frame(false))
                .clicked()
            {
                action.cycle_repeat = true;
            }
        });
    }

    fn render_lcd(
        ui: &mut Ui,
        player: &AudioPlayer,
        current_song: Option<&Song>,
        action: &mut PlayerBarAction,
    ) {
        ui.horizontal(|ui| {
            // 34px Artwork Box
            let (art_rect, art_resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
            ui.painter().rect_filled(art_rect, Rounding::same(6.0), BitChordTheme::SURFACE_ACTIVE);
            ui.painter().rect_stroke(
                art_rect,
                Rounding::same(6.0),
                Stroke::new(1.0_f32, BitChordTheme::GLASS_BORDER),
            );

            ui.painter().text(
                art_rect.center(),
                Align2::CENTER_CENTER,
                "🎵",
                FontId::proportional(16.0),
                BitChordTheme::KEY_CYAN,
            );

            if art_resp.clicked() {
                action.toggle_fullscreen = true;
            }

            ui.add_space(8.0);

            // Title + Artist + Scrubber
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(song) = current_song {
                        let title_cut = if song.title.len() > 22 {
                            format!("{}...", &song.title[..20])
                        } else {
                            song.title.clone()
                        };
                        ui.label(
                            RichText::new(title_cut)
                                .size(12.5)
                                .strong()
                                .color(BitChordTheme::TEXT_PRIMARY),
                        );
                        ui.label(
                            RichText::new("•")
                                .size(11.0)
                                .color(BitChordTheme::TEXT_MUTED),
                        );
                        let artist_cut = if song.artist.len() > 18 {
                            format!("{}...", &song.artist[..16])
                        } else {
                            song.artist.clone()
                        };
                        ui.label(
                            RichText::new(artist_cut)
                                .size(11.5)
                                .color(BitChordTheme::TEXT_SECONDARY),
                        );
                    } else {
                        ui.label(
                            RichText::new("BitChord")
                                .size(12.5)
                                .strong()
                                .color(BitChordTheme::TEXT_PRIMARY),
                        );
                    }
                });

                // Scrubber Seekbar
                let elapsed_ms = player.elapsed_ms();
                let duration_ms = current_song
                    .map(|s| (s.duration_seconds as u64) * 1000)
                    .unwrap_or(180_000)
                    .max(1000);

                ui.horizontal(|ui| {
                    let scrubber_width = (ui.available_width() - 40.0).max(60.0);
                    let (track_rect, track_resp) =
                        ui.allocate_exact_size(vec2(scrubber_width, 10.0), Sense::click_and_drag());

                    let progress = (elapsed_ms as f32 / duration_ms as f32).clamp(0.0, 1.0);
                    let bar_y = track_rect.center().y;

                    ui.painter().line_segment(
                        [pos2(track_rect.min.x, bar_y), pos2(track_rect.max.x, bar_y)],
                        Stroke::new(2.5_f32, BitChordTheme::SURFACE_ACTIVE),
                    );

                    let filled_x = track_rect.min.x + scrubber_width * progress;
                    ui.painter().line_segment(
                        [pos2(track_rect.min.x, bar_y), pos2(filled_x, bar_y)],
                        Stroke::new(2.5_f32, BitChordTheme::KEY_CYAN),
                    );

                    ui.painter().circle_filled(pos2(filled_x, bar_y), 4.0, Color32::WHITE);

                    if track_resp.clicked() || track_resp.dragged() {
                        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                            let ratio = ((mouse_pos.x - track_rect.min.x) / scrubber_width).clamp(0.0, 1.0);
                            action.seek_to_ms = Some((ratio * duration_ms as f32) as u64);
                        }
                    }

                    ui.label(
                        RichText::new(format_time_ms(elapsed_ms))
                            .size(10.0)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                });
            });
        });
    }

    fn render_right_actions(
        ui: &mut Ui,
        player: &AudioPlayer,
        _stats: &AudioQualityStats,
        lyrics_open: bool,
        queue_open: bool,
        is_liked: bool,
        action: &mut PlayerBarAction,
    ) {
        ui.horizontal_centered(|ui| {
            // Like Heart
            let heart_color = if is_liked {
                BitChordTheme::KEY_RED
            } else {
                BitChordTheme::TEXT_MUTED
            };
            if ui
                .add(egui::Button::new(RichText::new("♥").size(15.0).color(heart_color)).frame(false))
                .clicked()
            {
                action.toggle_like = true;
            }

            ui.add_space(2.0);

            // Lyrics Toggle
            let lyrics_color = if lyrics_open {
                BitChordTheme::KEY_CYAN
            } else {
                BitChordTheme::TEXT_MUTED
            };
            if ui
                .add(egui::Button::new(RichText::new("💬").size(14.0).color(lyrics_color)).frame(false))
                .on_hover_text("Lyrics")
                .clicked()
            {
                action.toggle_lyrics = true;
            }

            ui.add_space(2.0);

            // Queue Toggle
            let queue_color = if queue_open {
                BitChordTheme::KEY_CYAN
            } else {
                BitChordTheme::TEXT_MUTED
            };
            if ui
                .add(egui::Button::new(RichText::new("📑").size(14.0).color(queue_color)).frame(false))
                .on_hover_text("Up Next")
                .clicked()
            {
                action.toggle_queue = true;
            }

            ui.add_space(2.0);

            // Fullscreen
            if ui
                .add(egui::Button::new(RichText::new("⛶").size(14.0).color(BitChordTheme::TEXT_MUTED)).frame(false))
                .on_hover_text("Fullscreen Lyrics")
                .clicked()
            {
                action.toggle_fullscreen = true;
            }

            ui.add_space(4.0);

            // Volume Slider
            ui.label(RichText::new("🔊").size(12.0).color(BitChordTheme::TEXT_MUTED));
            let mut vol = player.volume();
            let vol_slider = egui::Slider::new(&mut vol, 0.0..=1.0)
                .show_value(false)
                .step_by(0.01);
            let resp = ui.add_sized(vec2(44.0, 12.0), vol_slider);
            if resp.changed() {
                player.set_volume(vol);
            }
        });
    }
}

pub fn format_time_ms(ms: u64) -> String {
    let total_sec = ms / 1000;
    let min = total_sec / 60;
    let sec = total_sec % 60;
    format!("{:02}:{:02}", min, sec)
}
