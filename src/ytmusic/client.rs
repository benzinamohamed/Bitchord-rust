use crate::models::{AudioQualityStats, HomeShelf, SearchResult, Song};
use crate::ytmusic::parser::{
    parse_home_shelves, parse_lyrics_browse_id, parse_lyrics_text, parse_next_queue,
    parse_search_results,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use std::error::Error;

const BASE_URL: &str = "https://music.youtube.com/youtubei/v1";
const WEB_REMIX_VERSION: &str = "1.20250101.01.00";
const WEB_REMIX_CLIENT_ID: &str = "67";

#[derive(Clone)]
pub struct YtMusicClient {
    client: reqwest::Client,
}

impl Default for YtMusicClient {
    fn default() -> Self {
        Self::new()
    }
}

impl YtMusicClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.9"),
        );
        headers.insert(
            "X-YouTube-Client-Name",
            HeaderValue::from_static(WEB_REMIX_CLIENT_ID),
        );
        headers.insert(
            "X-YouTube-Client-Version",
            HeaderValue::from_static(WEB_REMIX_VERSION),
        );
        headers.insert("Origin", HeaderValue::from_static("https://music.youtube.com"));
        headers.insert("Referer", HeaderValue::from_static("https://music.youtube.com/"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    fn default_context() -> Value {
        json!({
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": WEB_REMIX_VERSION,
                "hl": "en",
                "gl": "US"
            }
        })
    }

    pub async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/search?prettyPrint=false", BASE_URL);
        let body = json!({
            "context": Self::default_context(),
            "query": query
        });

        let resp: Value = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(parse_search_results(&resp, query))
    }

    pub async fn browse_home(&self) -> Result<Vec<HomeShelf>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/browse?prettyPrint=false", BASE_URL);
        let body = json!({
            "context": Self::default_context(),
            "browseId": "FEmusic_home"
        });

        let resp: Value = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(parse_home_shelves(&resp))
    }

    pub async fn get_up_next(&self, video_id: &str) -> Result<Vec<Song>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/next?prettyPrint=false", BASE_URL);
        let body = json!({
            "context": Self::default_context(),
            "videoId": video_id,
            "isAudioOnly": true
        });

        let resp: Value = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(parse_next_queue(&resp))
    }

    pub async fn get_lyrics(&self, video_id: &str) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        // Step 1: Query /next to get Lyrics tab browseId
        let next_url = format!("{}/next?prettyPrint=false", BASE_URL);
        let next_body = json!({
            "context": Self::default_context(),
            "videoId": video_id
        });

        let next_resp: Value = self
            .client
            .post(&next_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&next_body)
            .send()
            .await?
            .json()
            .await?;

        let browse_id = match parse_lyrics_browse_id(&next_resp) {
            Some(b) => b,
            None => return Ok(None),
        };

        // Step 2: Browse the lyrics shelf
        let browse_url = format!("{}/browse?prettyPrint=false", BASE_URL);
        let browse_body = json!({
            "context": Self::default_context(),
            "browseId": browse_id
        });

        let browse_resp: Value = self
            .client
            .post(&browse_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&browse_body)
            .send()
            .await?
            .json()
            .await?;

        Ok(parse_lyrics_text(&browse_resp))
    }

    pub async fn get_stream_url(
        &self,
        video_id: &str,
    ) -> Result<(String, AudioQualityStats), Box<dyn Error + Send + Sync>> {
        // Use ANDROID_VR client context to get direct audio streams with 0 cipher/signature friction
        let player_url = format!("{}/player", BASE_URL);
        let body = json!({
            "context": {
                "client": {
                    "clientName": "ANDROID_VR",
                    "clientVersion": "1.61.48",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id
        });

        let resp: Value = self
            .client
            .post(&player_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let formats = resp
            .pointer("/streamingData/adaptiveFormats")
            .and_then(|f| f.as_array())
            .ok_or("No adaptive formats found")?;

        // Find best audio format (Opus 251 or AAC 140)
        let mut best_url = None;
        let mut stats = AudioQualityStats::default();

        for f in formats {
            let mime = f.get("mimeType").and_then(|m| m.as_str()).unwrap_or("");
            if !mime.starts_with("audio/") {
                continue;
            }

            if let Some(url) = f.get("url").and_then(|u| u.as_str()) {
                let itag = f.get("itag").and_then(|i| i.as_u64()).unwrap_or(0);
                let bitrate = f.get("bitrate").and_then(|b| b.as_u64()).unwrap_or(128000) as u32 / 1000;
                let sample_rate = f.get("audioSampleRate").and_then(|s| s.as_str()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(48000);

                if itag == 251 {
                    // Opus 160kbps (best standard)
                    best_url = Some(url.to_string());
                    stats = AudioQualityStats {
                        codec: "Opus (WebM)".to_string(),
                        bitrate_kbps: bitrate,
                        sample_rate_hz: sample_rate,
                        bit_depth: "24-bit (BitChord Hi-Res)".to_string(),
                        is_lossless: false,
                    };
                    break;
                } else if itag == 140 && best_url.is_none() {
                    // AAC 128kbps fallback
                    best_url = Some(url.to_string());
                    stats = AudioQualityStats {
                        codec: "AAC (m4a)".to_string(),
                        bitrate_kbps: bitrate,
                        sample_rate_hz: sample_rate,
                        bit_depth: "16-bit".to_string(),
                        is_lossless: false,
                    };
                } else if best_url.is_none() {
                    best_url = Some(url.to_string());
                }
            }
        }

        if let Some(url) = best_url {
            Ok((url, stats))
        } else {
            Err("No direct stream URL available for track".into())
        }
    }
}
