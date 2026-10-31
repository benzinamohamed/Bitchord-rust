use crate::models::{AudioQualityStats, PlaybackStatus, RepeatMode, Song};
use crate::player::AudioPlayer;
use crate::ui::theme::BitChordTheme;
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, FontId, Rect, Response, RichText, Rounding, Sense,
    Stroke, Ui,
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

        let bar_height = 82.0;
        let margin_x = 20.0;
        let width = (ui.available_width() - margin_x * 2.0).clamp(300.0, 1100.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), bar_height), Sense::hover());
        let pill_rect = Rect::from_center_size(rect.center(), vec2(width, bar_height - 10.0));

        // Draw Frosted Glass Background
        ui.painter().rect_filled(pill_rect, Rounding::same(18.0), BitChordTheme::GLASS_PILL);
        ui.painter().rect_stroke(pill_rect, Rounding::same(18.0), BitChordTheme::glass_stroke());

        // Left Section: Song Artwork & Details
        let left_rect = Rect::from_min_size(
            pill_rect.min + vec2(14.0, 10.0),
            vec2(260.0, pill_rect.height() - 20.0),
        );
        ui.allocate_ui_at_rect(left_rect, |left_ui| {
            Self::render_song_info(left_ui, current_song, is_liked, &mut action);
        });

        // Center Section: Transport Controls & Scrubber
        let center_width = (pill_rect.width() - 560.0).max(280.0);
        let center_rect = Rect::from_center_size(
            pill_rect.center(),
            vec2(center_width, pill_rect.height() - 12.0),
        );
        ui.allocate_ui_at_rect(center_rect, |center_ui| {
            Self::render_center_controls(
                center_ui,
                player,
                current_song,
                repeat_mode,
                shuffle,
                &mut action,
            );
        });

        // Right Section: Volume & Utilities
        let right_rect = Rect::from_min_size(
            pos2(pill_rect.max.x - 260.0, pill_rect.min.y + 10.0),
            vec2(246.0, pill_rect.height() - 20.0),
        );
        ui.allocate_ui_at_rect(right_rect, |right_ui| {
            Self::render_right_controls(
                right_ui,
                player,
                stats,
                lyrics_open,
                queue_open,
                &mut action,
            );
        });

        action
    }

    fn render_song_info(
        ui: &mut Ui,
        current_song: Option<&Song>,
        is_liked: bool,
        action: &mut PlayerBarAction,
    ) {
        ui.horizontal_centered(|ui| {
            // Artwork Thumbnail Placeholder / Squircle
            let (thumb_rect, thumb_resp) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::click());
            ui.painter().rect_filled(thumb_rect, Rounding::same(10.0), BitChordTheme::SURFACE);
            ui.painter().rect_stroke(
                thumb_rect,
                Rounding::same(10.0),
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
            );

            // Draw music icon or miniature artwork
            ui.painter().text(
                thumb_rect.center(),
                Align2::CENTER_CENTER,
                "🎵",
                FontId::proportional(22.0),
                BitChordTheme::ACCENT_CYAN,
            );

            if thumb_resp.clicked() {
                action.toggle_fullscreen = true;
            }

            ui.add_space(8.0);

            // Title & Artist Column
            ui.vertical(|ui| {
                ui.add_space(6.0);
                if let Some(song) = current_song {
                    let title_text = if song.title.len() > 24 {
                        format!("{}...", &song.title[..22])
                    } else {
                        song.title.clone()
                    };
                    ui.label(
                        RichText::new(title_text)
                            .size(13.5)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );

                    let artist_text = if song.artist.len() > 26 {
                        format!("{}...", &song.artist[..24])
                    } else {
                        song.artist.clone()
                    };
                    ui.label(
                        RichText::new(artist_text)
                            .size(12.0)
                            .color(BitChordTheme::TEXT_SECONDARY),
                    );
                } else {
                    ui.label(
                        RichText::new("BitChord Ready")
                            .size(13.5)
                            .strong()
                            .color(BitChordTheme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("Select a track to play")
                            .size(12.0)
                            .color(BitChordTheme::TEXT_MUTED),
                    );
                }
            });

            ui.add_space(4.0);

            // Favorite / Like Heart Button
            let heart_color = if is_liked {
                BitChordTheme::ACCENT_PINK
            } else {
                BitChordTheme::TEXT_MUTED
            };
            if ui
                .add(egui::Button::new(RichText::new("♥").size(16.0).color(heart_color)).frame(false))
                .clicked()
            {
                action.toggle_like = true;
            }
        });
    }

    fn render_center_controls(
        ui: &mut Ui,
        player: &AudioPlayer,
        current_song: Option<&Song>,
        repeat_mode: RepeatMode,
        shuffle: bool,
        action: &mut PlayerBarAction,
    ) {
        let status = player.status();
        let is_playing = status == PlaybackStatus::Playing;
        let is_buffering = status == PlaybackStatus::Buffering;

        ui.vertical_centered(|ui| {
            // Upper Row: Transport Buttons
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 90.0);

                // Shuffle
                let shuffle_color = if shuffle {
                    BitChordTheme::ACCENT_CYAN
                } else {
                    BitChordTheme::TEXT_SECONDARY
                };
                if ui
                    .add(egui::Button::new(RichText::new("🔀").size(15.0).color(shuffle_color)).frame(false))
                    .clicked()
                {
                    action.toggle_shuffle = true;
                }

                ui.add_space(8.0);

                // Previous
                if ui
                    .add(egui::Button::new(RichText::new("⏮").size(18.0).color(BitChordTheme::TEXT_PRIMARY)).frame(false))
                    .clicked()
                {
                    action.previous = true;
                }

                ui.add_space(8.0);

                // Play / Pause Circle
                let (play_rect, play_resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
                let play_bg = if is_playing {
                    BitChordTheme::ACCENT_CYAN
                } else {
                    BitChordTheme::SURFACE_ACTIVE
                };
                ui.painter().circle_filled(play_rect.center(), 16.0, play_bg);

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
                    FontId::proportional(15.0),
                    icon_color,
                );

                if play_resp.clicked() {
                    action.toggle_play = true;
                }

                ui.add_space(8.0);

                // Next
                if ui
                    .add(egui::Button::new(RichText::new("⏭").size(18.0).color(BitChordTheme::TEXT_PRIMARY)).frame(false))
                    .clicked()
                {
                    action.next = true;
                }

                ui.add_space(8.0);

                // Repeat
                let repeat_color = match repeat_mode {
                    RepeatMode::Off => BitChordTheme::TEXT_SECONDARY,
                    RepeatMode::All => BitChordTheme::ACCENT_CYAN,
                    RepeatMode::One => BitChordTheme::ACCENT_PINK,
                };
                let repeat_icon = if repeat_mode == RepeatMode::One {
                    "🔂"
                } else {
                    "🔁"
                };
                if ui
                    .add(egui::Button::new(RichText::new(repeat_icon).size(15.0).color(repeat_color)).frame(false))
                    .clicked()
                {
                    action.cycle_repeat = true;
                }
            });

            ui.add_space(4.0);

            // Lower Row: Scrubber seekbar
            let elapsed_ms = player.elapsed_ms();
            let duration_ms = current_song
                .map(|s| (s.duration_seconds as u64) * 1000)
                .unwrap_or(180_000)
                .max(1000);

            ui.horizontal(|ui| {
                // Elapsed time label
                ui.label(
                    RichText::new(format_time_ms(elapsed_ms))
                        .size(11.0)
                        .color(BitChordTheme::TEXT_MUTED),
                );

                // Interactive Scrubber track
                let track_width = (ui.available_width() - 44.0).max(80.0);
                let (track_rect, track_resp) =
                    ui.allocate_exact_size(vec2(track_width, 12.0), Sense::click_and_drag());

                let progress = (elapsed_ms as f32 / duration_ms as f32).clamp(0.0, 1.0);
                let bar_y = track_rect.center().y;
                let bg_stroke = Stroke::new(3.5, BitChordTheme::SURFACE_ACTIVE);
                let active_stroke = Stroke::new(3.5, BitChordTheme::ACCENT_CYAN);

                // Background track
                ui.painter().line_segment(
                    [pos2(track_rect.min.x, bar_y), pos2(track_rect.max.x, bar_y)],
                    bg_stroke,
                );

                // Filled active portion
                let filled_x = track_rect.min.x + track_width * progress;
                ui.painter().line_segment(
                    [pos2(track_rect.min.x, bar_y), pos2(filled_x, bar_y)],
                    active_stroke,
                );

                // Knob
                ui.painter().circle_filled(pos2(filled_x, bar_y), 5.0, Color32::WHITE);

                if track_resp.clicked() || track_resp.dragged() {
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let click_ratio =
                            ((mouse_pos.x - track_rect.min.x) / track_width).clamp(0.0, 1.0);
                        let target_ms = (click_ratio * duration_ms as f32) as u64;
                        action.seek_to_ms = Some(target_ms);
                    }
                }

                // Duration label
                ui.label(
                    RichText::new(format_time_ms(duration_ms))
                        .size(11.0)
                        .color(BitChordTheme::TEXT_MUTED),
                );
            });
        });
    }

    fn render_right_controls(
        ui: &mut Ui,
        player: &AudioPlayer,
        stats: &AudioQualityStats,
        lyrics_open: bool,
        queue_open: bool,
        action: &mut PlayerBarAction,
    ) {
        ui.horizontal_centered(|ui| {
            // Lossless / Hi-Res Audio Badge
            let badge_text = if stats.bitrate_kbps > 0 {
                format!("{}k", stats.bitrate_kbps)
            } else {
                "Hi-Res".to_string()
            };
            let (badge_rect, _) = ui.allocate_exact_size(vec2(44.0, 18.0), Sense::hover());
            ui.painter().rect_filled(badge_rect, Rounding::same(4.0), BitChordTheme::SURFACE_ACTIVE);
            ui.painter().text(
                badge_rect.center(),
                Align2::CENTER_CENTER,
                &badge_text,
                FontId::proportional(9.5),
                BitChordTheme::ACCENT_CYAN,
            );

            ui.add_space(8.0);

            // Lyrics Toggle
            let lyrics_color = if lyrics_open {
                BitChordTheme::ACCENT_CYAN
            } else {
                BitChordTheme::TEXT_SECONDARY
            };
            if ui
                .add(egui::Button::new(RichText::new("💬").size(15.0).color(lyrics_color)).frame(false))
                .on_hover_text("Time-Synced Lyrics")
                .clicked()
            {
                action.toggle_lyrics = true;
            }

            ui.add_space(6.0);

            // Queue Drawer Toggle
            let queue_color = if queue_open {
                BitChordTheme::ACCENT_CYAN
            } else {
                BitChordTheme::TEXT_SECONDARY
            };
            if ui
                .add(egui::Button::new(RichText::new("📑").size(15.0).color(queue_color)).frame(false))
                .on_hover_text("Play Queue")
                .clicked()
            {
                action.toggle_queue = true;
            }

            ui.add_space(6.0);

            // Fullscreen Now Playing Expand
            if ui
                .add(egui::Button::new(RichText::new("⛶").size(16.0).color(BitChordTheme::TEXT_SECONDARY)).frame(false))
                .on_hover_text("Immersive Fullscreen Lyrics")
                .clicked()
            {
                action.toggle_fullscreen = true;
            }

            ui.add_space(8.0);

            // Volume Icon + Slider
            ui.label(RichText::new("🔊").size(14.0).color(BitChordTheme::TEXT_MUTED));
            let mut vol = player.volume();
            let vol_slider = egui::Slider::new(&mut vol, 0.0..=1.0)
                .show_value(false)
                .step_by(0.01);
            let resp = ui.add_sized(vec2(60.0, 14.0), vol_slider);
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
