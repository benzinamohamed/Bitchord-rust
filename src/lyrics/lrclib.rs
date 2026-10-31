use crate::models::LyricLine;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use std::error::Error;

const LRCLIB_BASE: &str = "https://lrclib.net/api";
const AGENT: &str = "BitChord-Rust (https://github.com/kushagrasinghx/BitChord)";

pub struct LyricsEngine {
    client: reqwest::Client,
    lrc_regex: Regex,
}

impl Default for LyricsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LyricsEngine {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(AGENT));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let lrc_regex = Regex::new(r"\[(\d{1,2}):(\d{2})(?:\.(\d{2,3}))?\](.*)").unwrap();

        Self { client, lrc_regex }
    }

    pub async fn fetch_synced_lyrics(
        &self,
        title: &str,
        artist: &str,
        duration_seconds: u32,
    ) -> Result<Vec<LyricLine>, Box<dyn Error + Send + Sync>> {
        let clean_title = clean_track_title(title);
        let clean_artist = clean_track_title(artist);

        // Try exact match first
        let mut url = format!(
            "{}/get?track_name={}&artist_name={}",
            LRCLIB_BASE,
            urlencoding::encode(&clean_title),
            urlencoding::encode(&clean_artist)
        );

        if duration_seconds > 0 {
            url.push_str(&format!("&duration={}", duration_seconds));
        }

        if let Ok(resp) = self.client.get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(val) = resp.json::<Value>().await {
                    if let Some(synced) = val.get("syncedLyrics").and_then(|s| s.as_str()) {
                        let parsed = self.parse_lrc(synced);
                        if !parsed.is_empty() {
                            return Ok(parsed);
                        }
                    }
                }
            }
        }

        // Try search fallback
        let search_url = format!(
            "{}/search?track_name={}&artist_name={}",
            LRCLIB_BASE,
            urlencoding::encode(&clean_title),
            urlencoding::encode(&clean_artist)
        );

        if let Ok(resp) = self.client.get(&search_url).send().await {
            if resp.status().is_success() {
                if let Ok(arr) = resp.json::<Value>().await {
                    if let Some(list) = arr.as_array() {
                        for item in list {
                            if let Some(synced) = item.get("syncedLyrics").and_then(|s| s.as_str()) {
                                let parsed = self.parse_lrc(synced);
                                if !parsed.is_empty() {
                                    return Ok(parsed);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("No synced lyrics found on LRCLIB".into())
    }

    pub fn parse_lrc(&self, lrc: &str) -> Vec<LyricLine> {
        let mut lines = Vec::new();
        for raw_line in lrc.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(caps) = self.lrc_regex.captures(trimmed) {
                let minutes: u64 = caps[1].parse().unwrap_or(0);
                let seconds: u64 = caps[2].parse().unwrap_or(0);
                let fraction: u64 = caps.get(3).map_or(0, |m| {
                    let s = m.as_str();
                    let val: u64 = s.parse().unwrap_or(0);
                    if s.len() == 2 {
                        val * 10
                    } else {
                        val
                    }
                });

                let start_ms = minutes * 60_000 + seconds * 1_000 + fraction;
                let text = caps[4].trim().to_string();

                if !text.is_empty() {
                    lines.push(LyricLine { start_ms, text });
                }
            }
        }

        lines.sort_by_key(|l| l.start_ms);
        lines
    }

    /// Generates timed lines from plain text lyrics spread across track duration
    pub fn interpolate_plain_lyrics(plain: &str, duration_seconds: u32) -> Vec<LyricLine> {
        let text_lines: Vec<&str> = plain
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        if text_lines.is_empty() {
            return Vec::new();
        }

        let dur_ms = if duration_seconds > 0 {
            (duration_seconds as u64) * 1000
        } else {
            180_000 // default 3 minutes
        };

        // Lead-in time of 5 seconds, ending 5 seconds before end
        let start_offset = 5000.min(dur_ms / 10);
        let usable_dur = dur_ms.saturating_sub(start_offset + 5000);
        let step = usable_dur / (text_lines.len() as u64).max(1);

        text_lines
            .into_iter()
            .enumerate()
            .map(|(i, t)| LyricLine {
                start_ms: start_offset + (i as u64) * step,
                text: t.to_string(),
            })
            .collect()
    }
}

fn clean_track_title(title: &str) -> String {
    // Strip (Official Video), [MV], (Remastered), etc.
    let re = Regex::new(r"(?i)\s*[\(\[](official|video|audio|remastered|lyric|hd|4k|mv).*?[\)\]]").unwrap();
    let cleaned = re.replace_all(title, "");
    cleaned.trim().to_string()
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
