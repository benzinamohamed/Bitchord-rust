use crate::lyrics::lrclib::LyricsEngine;
use crate::models::{AudioQualityStats, HomeShelf, PlaybackStatus, SearchResult, Song};
use crate::player::audio::AudioPlayer;
use crate::ytmusic::client::YtMusicClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub player: Arc<AudioPlayer>,
    pub ytmusic: Arc<YtMusicClient>,
    pub lyrics: Arc<LyricsEngine>,
    pub http: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(rename = "bodyEncoding")]
    pub body_encoding: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStatusResponse {
    pub status: String,
    pub elapsed_sec: u64,
    pub elapsed_ms: u64,
    pub duration_seconds: u32,
    pub volume: f32,
    pub current_song: Option<Song>,
    pub stats: AudioQualityStats,
}

#[tauri::command]
pub fn log_from_js(msg: String) {
    eprintln!("[JS] {}", msg);
}

#[tauri::command]
pub async fn bridge_call(
    state: State<'_, AppState>,
    method: String,
    params: Value,
) -> Result<Value, String> {
    eprintln!("[TAURI BRIDGE] method: {}", method);
    match method.as_str() {
        "ping" => Ok(json!({
            "version": "1.0.0",
            "protocol": 1,
            "browser": "chromium",
            "capabilities": ["yt", "yt.auth", "yt.stats", "addon", "lyrics", "spotify", "discord"]
        })),

        "yt.fetch" | "lyrics.fetch" | "addon.fetch" => {
            let url_str = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing 'url' parameter".to_string())?;

            let method_str = params
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET")
                .to_uppercase();

            let mut req = match method_str.as_str() {
                "POST" => state.http.post(url_str),
                "HEAD" => state.http.head(url_str),
                _ => state.http.get(url_str),
            };

            // Set browser-like headers
            let mut headers = HeaderMap::new();
            headers.insert(
                USER_AGENT,
                HeaderValue::from_static(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
                ),
            );

            if url_str.contains("youtube.com") {
                headers.insert("Origin", HeaderValue::from_static("https://music.youtube.com"));
                headers.insert("Referer", HeaderValue::from_static("https://music.youtube.com/"));
                headers.insert("X-YouTube-Client-Name", HeaderValue::from_static("67"));
                headers.insert("X-YouTube-Client-Version", HeaderValue::from_static("1.20250101.01.00"));
            }

            if let Some(h_obj) = params.get("headers").and_then(|v| v.as_object()) {
                for (k, v) in h_obj {
                    if let Some(val_str) = v.as_str() {
                        if let (Ok(h_name), Ok(h_val)) = (
                            HeaderName::from_bytes(k.as_bytes()),
                            HeaderValue::from_str(val_str),
                        ) {
                            headers.insert(h_name, h_val);
                        }
                    }
                }
            }

            req = req.headers(headers);

            if let Some(body_val) = params.get("body") {
                if let Some(body_str) = body_val.as_str() {
                    req = req.body(body_str.to_string());
                }
            }

            let resp = req.send().await.map_err(|e| format!("HTTP request failed: {}", e))?;
            let status = resp.status().as_u16();
            let final_url = resp.url().to_string();

            let mut resp_headers = HashMap::new();
            for (k, v) in resp.headers() {
                if let Ok(v_str) = v.to_str() {
                    resp_headers.insert(k.as_str().to_string(), v_str.to_string());
                }
            }

            let body_bytes = resp.bytes().await.map_err(|e| format!("Reading body failed: {}", e))?;
            let body_text = String::from_utf8_lossy(&body_bytes).to_string();

            let res = FetchResponse {
                status,
                headers: resp_headers,
                body: body_text,
                body_encoding: "text".to_string(),
                url: final_url,
            };

            serde_json::to_value(res).map_err(|e| e.to_string())
        }

        "lyrics.permissions" => {
            let origins = params
                .get("origins")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            Ok(json!({ "granted": origins }))
        }

        "lyrics.requestPermissions" => Ok(json!({})),

        "yt.authState" => Ok(json!({
            "browserSignedIn": false,
            "connected": false,
            "signedIn": false,
            "accounts": [],
            "active": null,
            "problem": null
        })),

        "yt.selectChannel" | "yt.setConnected" => Ok(json!({
            "browserSignedIn": false,
            "connected": false,
            "signedIn": false,
            "accounts": [],
            "active": null,
            "problem": null
        })),

        "spotify.authState" => Ok(json!({
            "browserSignedIn": false,
            "connected": false,
            "signedIn": false,
            "user": null,
            "problem": null
        })),

        "discord.state" => Ok(json!({
            "available": true,
            "connected": false,
            "enabled": false,
            "user": null,
            "gateway": "off",
            "problem": null
        })),

        "discord.presence" => Ok(json!({})),

        _ => Err(format!("Unsupported bridge method: {}", method)),
    }
}

#[tauri::command]
pub async fn native_play(
    state: State<'_, AppState>,
    song: Song,
) -> Result<AudioQualityStats, String> {
    eprintln!("[TAURI PLAY] native_play called for: '{}' by '{}' (id: {})", song.title, song.artist, song.video_id);
    let video_id = song.video_id.clone();
    state.player.set_loading(true);

    let (stream_url, stats, user_agent) = state
        .ytmusic
        .get_stream_url(&video_id)
        .await
        .map_err(|e| format!("Failed to resolve stream: {}", e))?;

    state
        .player
        .play_stream(stream_url, song, stats.clone(), user_agent, 0)?;

    Ok(stats)
}

#[tauri::command]
pub fn native_toggle_play(state: State<'_, AppState>) {
    state.player.toggle_play();
}

#[tauri::command]
pub fn native_pause(state: State<'_, AppState>) {
    if state.player.status() == PlaybackStatus::Playing {
        state.player.toggle_play();
    }
}

#[tauri::command]
pub fn native_resume(state: State<'_, AppState>) {
    if state.player.status() == PlaybackStatus::Paused {
        state.player.toggle_play();
    }
}

#[tauri::command]
pub fn native_seek(state: State<'_, AppState>, seconds: u64) {
    state.player.seek_to_sec(seconds);
}

#[tauri::command]
pub fn native_set_volume(state: State<'_, AppState>, volume: f32) {
    state.player.set_volume(volume);
}

#[tauri::command]
pub fn native_status(state: State<'_, AppState>) -> PlayerStatusResponse {
    let current_song = state.player.current_song();
    let duration_seconds = current_song.as_ref().map(|s| s.duration_seconds).unwrap_or(0);
    let status_str = match state.player.status() {
        PlaybackStatus::Playing => "playing",
        PlaybackStatus::Paused => "paused",
        PlaybackStatus::Buffering => "buffering",
        PlaybackStatus::Stopped => "idle",
    };

    PlayerStatusResponse {
        status: status_str.to_string(),
        elapsed_sec: state.player.elapsed_sec(),
        elapsed_ms: state.player.elapsed_ms(),
        duration_seconds,
        volume: state.player.volume(),
        current_song,
        stats: state.player.stats(),
    }
}

#[tauri::command]
pub async fn native_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<SearchResult, String> {
    state
        .ytmusic
        .search(&query)
        .await
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
pub async fn native_home(state: State<'_, AppState>) -> Result<Vec<HomeShelf>, String> {
    state
        .ytmusic
        .browse_home()
        .await
        .map_err(|e| format!("Get home failed: {}", e))
}

#[tauri::command]
pub async fn native_up_next(
    state: State<'_, AppState>,
    video_id: String,
) -> Result<Vec<Song>, String> {
    state
        .ytmusic
        .get_up_next(&video_id)
        .await
        .map_err(|e| format!("Get up next failed: {}", e))
}

#[tauri::command]
pub async fn native_lyrics(
    state: State<'_, AppState>,
    video_id: String,
    title: String,
    artist: String,
    duration_seconds: u32,
) -> Result<Value, String> {
    // 1. Try LRCLIB for word-by-word synced lyrics
    if let Ok(lines) = state
        .lyrics
        .fetch_synced_lyrics(&title, &artist, duration_seconds)
        .await
    {
        if !lines.is_empty() {
            return Ok(json!({
                "synced": true,
                "lines": lines,
            }));
        }
    }

    // 2. Fallback to YouTube Music lyrics
    if let Ok(Some(plain_text)) = state.ytmusic.get_lyrics(&video_id).await {
        return Ok(json!({
            "synced": false,
            "lines": [],
            "raw": plain_text
        }));
    }

    Ok(json!({
        "synced": false,
        "lines": [],
        "raw": null
    }))
}
