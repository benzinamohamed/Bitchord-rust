use crate::models::{Album, Artist, HomeShelf, SearchResult, Song};
use serde_json::Value;

pub fn parse_search_results(val: &Value, query: &str) -> SearchResult {
    let mut result = SearchResult {
        query: query.to_string(),
        top_result: None,
        songs: Vec::new(),
        albums: Vec::new(),
        artists: Vec::new(),
    };

    // Find sectionListRenderer contents
    let sections = val
        .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(|v| v.as_array());

    if let Some(sections) = sections {
        for section in sections {
            // Check for musicCardShelfRenderer (Top result)
            if let Some(card) = section.get("musicCardShelfRenderer") {
                if let Some(song) = parse_card_shelf(card) {
                    if result.top_result.is_none() {
                        result.top_result = Some(song.clone());
                        result.songs.push(song);
                    }
                }
            }

            // Check itemSectionRenderer
            if let Some(item_section) = section.get("itemSectionRenderer") {
                if let Some(contents) = item_section.get("contents").and_then(|c| c.as_array()) {
                    for item in contents {
                        if let Some(list_item) = item.get("musicResponsiveListItemRenderer") {
                            classify_and_add_list_item(list_item, &mut result);
                        }
                    }
                }
            }

            // Check musicShelfRenderer
            if let Some(music_shelf) = section.get("musicShelfRenderer") {
                if let Some(contents) = music_shelf.get("contents").and_then(|c| c.as_array()) {
                    for item in contents {
                        if let Some(list_item) = item.get("musicResponsiveListItemRenderer") {
                            classify_and_add_list_item(list_item, &mut result);
                        }
                    }
                }
            }
        }
    }

    result
}

fn parse_card_shelf(card: &Value) -> Option<Song> {
    let title = card.pointer("/title/runs/0/text")?.as_str()?.to_string();
    let video_id = card
        .pointer("/onTap/watchEndpoint/videoId")
        .or_else(|| card.pointer("/title/runs/0/navigationEndpoint/watchEndpoint/videoId"))
        .and_then(|v| v.as_str())?
        .to_string();

    let artist = card
        .pointer("/subtitle/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist")
        .to_string();

    let thumbnail_url = card
        .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    Some(Song {
        video_id,
        title,
        artist,
        thumbnail_url,
        duration_text: None,
        duration_seconds: 0,
        album_id: None,
        album_name: None,
        is_video: false,
    })
}

fn classify_and_add_list_item(item: &Value, result: &mut SearchResult) {
    let flex = match item.get("flexColumns").and_then(|f| f.as_array()) {
        Some(f) => f,
        None => return,
    };

    let col0_runs = flex.get(0).and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs")).and_then(|r| r.as_array());
    let title = col0_runs
        .and_then(|runs| runs.first())
        .and_then(|r| r.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    if title.is_empty() {
        return;
    }

    let video_id = col0_runs
        .and_then(|runs| runs.first())
        .and_then(|r| r.pointer("/navigationEndpoint/watchEndpoint/videoId"))
        .or_else(|| item.pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let col1_runs = flex.get(1).and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs")).and_then(|r| r.as_array());
    let mut item_type = String::new();
    let mut artist_name = String::new();
    let mut album_name = None;
    let mut duration_text = None;

    if let Some(runs) = col1_runs {
        let texts: Vec<&str> = runs.iter().filter_map(|r| r.get("text").and_then(|t| t.as_str())).collect();
        if !texts.is_empty() {
            item_type = texts[0].to_lowercase();
        }

        // Search for artist run
        for (i, &txt) in texts.iter().enumerate() {
            if txt == " • " || txt == "Song" || txt == "Video" || txt == "Artist" || txt == "Album" || txt == "Single" {
                continue;
            }
            if artist_name.is_empty() {
                artist_name = txt.to_string();
            } else if album_name.is_none() && !txt.contains(':') {
                album_name = Some(txt.to_string());
            } else if txt.contains(':') && duration_text.is_none() {
                duration_text = Some(txt.to_string());
            }
        }
    }

    let thumbnail_url = item
        .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    if let Some(vid) = video_id {
        // It's a playable track!
        let duration_seconds = parse_duration_seconds(duration_text.as_deref().unwrap_or(""));
        result.songs.push(Song {
            video_id: vid,
            title,
            artist: if artist_name.is_empty() { "Unknown Artist".to_string() } else { artist_name },
            thumbnail_url,
            duration_text,
            duration_seconds,
            album_id: None,
            album_name,
            is_video: item_type.contains("video"),
        });
    } else if item_type.contains("artist") {
        let browse_id = item
            .pointer("/navigationEndpoint/browseEndpoint/browseId")
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string();
        result.artists.push(Artist {
            browse_id,
            name: title,
            thumbnail_url,
            subscribers: col1_runs.and_then(|r| r.last()).and_then(|t| t.get("text")).and_then(|s| s.as_str()).map(|s| s.to_string()),
        });
    } else if item_type.contains("album") || item_type.contains("single") {
        let browse_id = item
            .pointer("/navigationEndpoint/browseEndpoint/browseId")
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string();
        result.albums.push(Album {
            browse_id,
            title,
            artist: artist_name,
            thumbnail_url,
            year: None,
        });
    }
}

pub fn parse_home_shelves(val: &Value) -> Vec<HomeShelf> {
    let mut shelves = Vec::new();
    let sections = val
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(|s| s.as_array());

    if let Some(sections) = sections {
        for sec in sections {
            let shelf_obj = sec.get("musicCarouselShelfRenderer").or_else(|| sec.get("musicShelfRenderer"));
            if let Some(shelf) = shelf_obj {
                let title = shelf
                    .pointer("/header/musicCarouselShelfBasicHeaderRenderer/title/runs/0/text")
                    .or_else(|| shelf.pointer("/title/runs/0/text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Trending")
                    .to_string();

                let mut songs = Vec::new();
                if let Some(contents) = shelf.get("contents").and_then(|c| c.as_array()) {
                    for item in contents {
                        if let Some(two_row) = item.get("musicTwoRowItemRenderer") {
                            if let Some(song) = parse_two_row_song(two_row) {
                                songs.push(song);
                            }
                        } else if let Some(responsive) = item.get("musicResponsiveListItemRenderer") {
                            let mut dummy = SearchResult::default();
                            classify_and_add_list_item(responsive, &mut dummy);
                            songs.extend(dummy.songs);
                        }
                    }
                }

                if !songs.is_empty() {
                    shelves.push(HomeShelf { title, items: songs });
                }
            }
        }
    }

    shelves
}

fn parse_two_row_song(item: &Value) -> Option<Song> {
    let title = item.pointer("/title/runs/0/text")?.as_str()?.to_string();
    let video_id = item
        .pointer("/navigationEndpoint/watchEndpoint/videoId")
        .or_else(|| item.pointer("/thumbnailOverlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId"))
        .and_then(|v| v.as_str())?
        .to_string();

    let artist = item
        .pointer("/subtitle/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Various Artists")
        .to_string();

    let thumbnail_url = item
        .pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail/thumbnails")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    Some(Song {
        video_id,
        title,
        artist,
        thumbnail_url,
        duration_text: None,
        duration_seconds: 0,
        album_id: None,
        album_name: None,
        is_video: false,
    })
}

pub fn parse_next_queue(val: &Value) -> Vec<Song> {
    let mut songs = Vec::new();
    let queue_items = val
        .pointer("/contents/singleColumnMusicWatchNextResultsRenderer/tabbedRenderer/watchNextTabbedResultsRenderer/tabs/0/tabRenderer/content/musicQueueRenderer/content/playlistPanelRenderer/contents")
        .and_then(|c| c.as_array());

    if let Some(items) = queue_items {
        for item in items {
            if let Some(renderer) = item.get("playlistPanelVideoRenderer") {
                if let (Some(vid), Some(title)) = (
                    renderer.get("videoId").and_then(|v| v.as_str()),
                    renderer.pointer("/title/runs/0/text").and_then(|t| t.as_str()),
                ) {
                    let artist = renderer
                        .pointer("/shortBylineText/runs/0/text")
                        .and_then(|a| a.as_str())
                        .unwrap_or("Artist")
                        .to_string();
                    let duration_text = renderer
                        .pointer("/lengthText/runs/0/text")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());
                    let duration_seconds = parse_duration_seconds(duration_text.as_deref().unwrap_or(""));
                    let thumbnail_url = renderer
                        .pointer("/thumbnail/thumbnails")
                        .and_then(|t| t.as_array())
                        .and_then(|a| a.last())
                        .and_then(|t| t.get("url"))
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string());

                    songs.push(Song {
                        video_id: vid.to_string(),
                        title: title.to_string(),
                        artist,
                        thumbnail_url,
                        duration_text,
                        duration_seconds,
                        album_id: None,
                        album_name: None,
                        is_video: false,
                    });
                }
            }
        }
    }

    songs
}

pub fn parse_lyrics_browse_id(val: &Value) -> Option<String> {
    let tabs = val
        .pointer("/contents/singleColumnMusicWatchNextResultsRenderer/tabbedRenderer/watchNextTabbedResultsRenderer/tabs")
        .and_then(|t| t.as_array())?;

    for tab in tabs {
        let title = tab.pointer("/tabRenderer/title")?.as_str()?;
        if title.eq_ignore_ascii_case("lyrics") {
            return tab
                .pointer("/tabRenderer/endpoint/browseEndpoint/browseId")
                .and_then(|b| b.as_str())
                .map(|s| s.to_string());
        }
    }
    None
}

pub fn parse_lyrics_text(val: &Value) -> Option<String> {
    val.pointer("/contents/sectionListRenderer/contents/0/musicDescriptionShelfRenderer/description/runs/0/text")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

fn parse_duration_seconds(dur: &str) -> u32 {
    let parts: Vec<&str> = dur.split(':').collect();
    if parts.len() == 2 {
        let min = parts[0].parse::<u32>().unwrap_or(0);
        let sec = parts[1].parse::<u32>().unwrap_or(0);
        min * 60 + sec
    } else if parts.len() == 3 {
        let hr = parts[0].parse::<u32>().unwrap_or(0);
        let min = parts[1].parse::<u32>().unwrap_or(0);
        let sec = parts[2].parse::<u32>().unwrap_or(0);
        hr * 3600 + min * 60 + sec
    } else {
        0
    }
}
