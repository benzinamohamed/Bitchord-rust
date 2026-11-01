use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Song {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub thumbnail_url: Option<String>,
    pub duration_text: Option<String>,
    pub duration_seconds: u32,
    pub album_id: Option<String>,
    pub album_name: Option<String>,
    pub is_video: bool,
}

impl Song {
    pub fn new(video_id: &str, title: &str, artist: &str) -> Self {
        Self {
            video_id: video_id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            thumbnail_url: None,
            duration_text: None,
            duration_seconds: 0,
            album_id: None,
            album_name: None,
            is_video: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Album {
    pub browse_id: String,
    pub title: String,
    pub artist: String,
    pub thumbnail_url: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
    pub browse_id: String,
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub subscribers: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub browse_id: String,
    pub title: String,
    pub author: Option<String>,
    pub track_count: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeShelf {
    pub title: String,
    pub items: Vec<Song>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub top_result: Option<Song>,
    pub songs: Vec<Song>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LyricLine {
    pub start_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Stopped,
    Buffering,
    Playing,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityStats {
    pub codec: String,
    pub bitrate_kbps: u32,
    pub sample_rate_hz: u32,
    pub bit_depth: String,
    pub is_lossless: bool,
}

impl Default for AudioQualityStats {
    fn default() -> Self {
        Self {
            codec: "Opus (WebM)".to_string(),
            bitrate_kbps: 160,
            sample_rate_hz: 48000,
            bit_depth: "24-bit".to_string(),
            is_lossless: false,
        }
    }
}
