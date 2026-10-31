use crate::models::{AudioQualityStats, HomeShelf, SearchResult, Song};
use crate::ytmusic::parser::{
    parse_home_shelves, parse_lyrics_browse_id, parse_lyrics_text, parse_next_queue,
    parse_search_results,
};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

const BASE_URL: &str = "https://music.youtube.com/youtubei/v1";
const WEB_BASE_URL: &str = "https://www.youtube.com/youtubei/v1";
const WEB_REMIX_VERSION: &str = "1.20250101.01.00";
const WEB_REMIX_CLIENT_ID: &str = "67";

const VISIONOS_UA: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
const ANDROID_VR_UA: &str =
    "com.google.android.apps.youtube.vr.oculus/1.65.10 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip";

#[derive(Clone)]
pub struct YtMusicClient {
    client: reqwest::Client,
    visitor_data: Arc<RwLock<Option<String>>>,
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

        Self {
            client,
            visitor_data: Arc::new(RwLock::new(None)),
        }
    }

    /// Fetches and caches the YouTube session visitor data from service worker bootstrap
    pub async fn ensure_visitor_data(&self) -> Option<String> {
        {
            let read = self.visitor_data.read().await;
            if let Some(ref v) = *read {
                return Some(v.clone());
            }
        }

        let resp = self
            .client
            .get("https://www.youtube.com/sw.js_data")
            .header(
                USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
            )
            .send()
            .await;

        if let Ok(r) = resp {
            if let Ok(text) = r.text().await {
                let re = Regex::new(r"Cg[A-Za-z0-9_%-]{40,}").unwrap();
                if let Some(mat) = re.find(&text) {
                    let v = mat.as_str().to_string();
                    let mut write = self.visitor_data.write().await;
                    *write = Some(v.clone());
                    return Some(v);
                }
            }
        }

        None
    }

    fn default_context(&self, visitor: Option<&str>) -> Value {
        let mut client_obj = json!({
            "clientName": "WEB_REMIX",
            "clientVersion": WEB_REMIX_VERSION,
            "hl": "en",
            "gl": "US"
        });
        if let Some(v) = visitor {
            client_obj["visitorData"] = json!(v);
        }
        json!({
            "client": client_obj
        })
    }

    pub async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn Error + Send + Sync>> {
        let visitor = self.ensure_visitor_data().await;
        let url = format!("{}/search?prettyPrint=false", BASE_URL);
        let body = json!({
            "context": self.default_context(visitor.as_deref()),
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
        let visitor = self.ensure_visitor_data().await;
        let url = format!("{}/browse?prettyPrint=false", BASE_URL);
        let body = json!({
            "context": self.default_context(visitor.as_deref()),
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

        let mut shelves = parse_home_shelves(&resp);

        // If home has only 1 shelf or is sparse, fetch charts to supplement
        if shelves.len() <= 1 {
            let charts_body = json!({
                "context": self.default_context(visitor.as_deref()),
                "browseId": "FEmusic_charts"
            });
            if let Ok(c_resp) = self.client.post(&url).header(CONTENT_TYPE, "application/json").json(&charts_body).send().await {
                if let Ok(c_json) = c_resp.json::<Value>().await {
                    let chart_shelves = parse_home_shelves(&c_json);
                    for s in chart_shelves {
                        if !shelves.iter().any(|existing| existing.title == s.title) {
                            shelves.push(s);
                        }
                    }
                }
            }
        }

        Ok(shelves)
    }

    pub async fn get_up_next(&self, video_id: &str) -> Result<Vec<Song>, Box<dyn Error + Send + Sync>> {
        let visitor = self.ensure_visitor_data().await;
        let url = format!("{}/next?prettyPrint=false", BASE_URL);
        let body = json!({
            "context": self.default_context(visitor.as_deref()),
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
        let visitor = self.ensure_visitor_data().await;
        let next_url = format!("{}/next?prettyPrint=false", BASE_URL);
        let next_body = json!({
            "context": self.default_context(visitor.as_deref()),
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

        let browse_url = format!("{}/browse?prettyPrint=false", BASE_URL);
        let browse_body = json!({
            "context": self.default_context(visitor.as_deref()),
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

    /// Resolves stream URL using VISIONOS direct stream resolution (as used by BitChord / InnerTubeX)
    /// Returns (stream_url, audio_stats, matching_user_agent)
    pub async fn get_stream_url(
        &self,
        video_id: &str,
    ) -> Result<(String, AudioQualityStats, String), Box<dyn Error + Send + Sync>> {
        let visitor = self.ensure_visitor_data().await;

        // Candidate clients: VISIONOS (primary direct URLs) and ANDROID_VR fallback
        let candidates = [
            ("VISIONOS", "1.02", VISIONOS_UA),
            ("ANDROID_VR", "1.65.10", ANDROID_VR_UA),
        ];

        let player_url = format!("{}/player", WEB_BASE_URL);

        for (client_name, client_ver, ua) in candidates {
            let mut client_obj = json!({
                "clientName": client_name,
                "clientVersion": client_ver,
                "hl": "en",
                "gl": "US"
            });
            if let Some(ref v) = visitor {
                client_obj["visitorData"] = json!(v);
            }

            let body = json!({
                "context": {
                    "client": client_obj
                },
                "videoId": video_id
            });

            let resp = self
                .client
                .post(&player_url)
                .header(CONTENT_TYPE, "application/json")
                .header(USER_AGENT, ua)
                .json(&body)
                .send()
                .await;

            let resp_json: Value = match resp {
                Ok(r) => match r.json().await {
                    Ok(j) => j,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            let formats = match resp_json
                .pointer("/streamingData/adaptiveFormats")
                .and_then(|f| f.as_array())
            {
                Some(f) => f,
                None => continue,
            };

            // Collect direct audio formats
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
                    let sample_rate = f
                        .get("audioSampleRate")
                        .and_then(|s| s.as_str())
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(48000);

                    if itag == 251 {
                        // Opus 160kbps (BitChord Hi-Res Master)
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
                return Ok((url, stats, ua.to_string()));
            }
        }

        Err("No playable audio stream available for track".into())
    }
}
