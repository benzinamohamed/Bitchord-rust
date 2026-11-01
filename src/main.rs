#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod lyrics;
mod models;
mod player;
mod ytmusic;

use commands::*;
use lyrics::lrclib::LyricsEngine;
use player::audio::AudioPlayer;
use std::sync::Arc;
use ytmusic::client::YtMusicClient;

fn main() {
    let player = Arc::new(AudioPlayer::new());
    let ytmusic = Arc::new(YtMusicClient::new());
    let lyrics = Arc::new(LyricsEngine::new());
    let http = reqwest::Client::builder()
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let state = AppState {
        player,
        ytmusic,
        lyrics,
        http,
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            log_from_js,
            bridge_call,
            native_play,
            native_toggle_play,
            native_pause,
            native_resume,
            native_seek,
            native_set_volume,
            native_status,
            native_search,
            native_home,
            native_up_next,
            native_lyrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running bitchord application");
}
