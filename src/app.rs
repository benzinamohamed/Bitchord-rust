use crate::lyrics::LyricsEngine;
use crate::models::{AudioQualityStats, HomeShelf, LyricLine, SearchResult, Song};
use crate::player::{AudioPlayer, PlaybackQueue};
use crate::ui::{
    AppSettings, BitChordTheme, ExploreView, HomeView, LibraryView, NavTab, NowPlayingView,
    PlayerBar, SearchView, SettingsView, Sidebar,
};
use crate::ytmusic::YtMusicClient;
use eframe::egui::{self, CentralPanel, Margin, SidePanel};
use std::sync::{mpsc, Arc, Mutex};
use tokio::runtime::Handle;

pub enum AppMessage {
    HomeLoaded(Vec<HomeShelf>),
    SearchLoaded(SearchResult),
    TrackReady {
        song: Song,
        url: String,
        stats: AudioQualityStats,
        user_agent: String,
        lyrics: Vec<LyricLine>,
        up_next: Vec<Song>,
    },
    LyricsLoaded(Vec<LyricLine>),
    Error(String),
}

pub struct BitChordApp {
    pub player: Arc<AudioPlayer>,
    pub queue: Arc<Mutex<PlaybackQueue>>,
    pub yt_client: YtMusicClient,
    pub lyrics_engine: Arc<LyricsEngine>,
    pub tokio_handle: Handle,

    // Channels
    pub tx: mpsc::Sender<AppMessage>,
    pub rx: mpsc::Receiver<AppMessage>,

    // UI Navigation State
    pub current_tab: NavTab,
    pub search_query: String,
    pub search_results: SearchResult,
    pub is_searching: bool,
    pub home_shelves: Vec<HomeShelf>,
    pub is_loading_home: bool,

    // Player & Lyrics State
    pub current_lyrics: Vec<LyricLine>,
    pub lyrics_open: bool,
    pub queue_open: bool,
    pub fullscreen_now_playing: bool,
    pub liked_songs: Vec<Song>,
    pub history: Vec<Song>,
    pub settings: AppSettings,
}

impl BitChordApp {
    pub fn new(cc: &eframe::CreationContext<'_>, handle: Handle) -> Self {
        // Configure egui visuals for Apple Music dark look
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BitChordTheme::BG_MAIN;
        visuals.window_fill = BitChordTheme::BG_DEEP;
        visuals.extreme_bg_color = BitChordTheme::SURFACE;
        cc.egui_ctx.set_visuals(visuals);

        let (tx, rx) = mpsc::channel();
        let player = Arc::new(AudioPlayer::new());
        let queue = Arc::new(Mutex::new(PlaybackQueue::new()));
        let yt_client = YtMusicClient::new();
        let lyrics_engine = Arc::new(LyricsEngine::new());

        let app = Self {
            player,
            queue,
            yt_client,
            lyrics_engine,
            tokio_handle: handle,
            tx,
            rx,
            current_tab: NavTab::Home,
            search_query: String::new(),
            search_results: SearchResult::default(),
            is_searching: false,
            home_shelves: Vec::new(),
            is_loading_home: true,
            current_lyrics: Vec::new(),
            lyrics_open: false,
            queue_open: false,
            fullscreen_now_playing: false,
            liked_songs: Vec::new(),
            history: Vec::new(),
            settings: AppSettings::default(),
        };

        // Load initial home recommendations
        app.load_home();

        app
    }

    pub fn load_home(&self) {
        let client = self.yt_client.clone();
        let tx = self.tx.clone();

        self.tokio_handle.spawn(async move {
            match client.browse_home().await {
                Ok(shelves) => {
                    let _ = tx.send(AppMessage::HomeLoaded(shelves));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to load home: {}", e)));
                }
            }
        });
    }

    pub fn trigger_search(&mut self) {
        let query = self.search_query.trim().to_string();
        if query.is_empty() {
            return;
        }

        self.is_searching = true;
        let client = self.yt_client.clone();
        let tx = self.tx.clone();

        self.tokio_handle.spawn(async move {
            match client.search(&query).await {
                Ok(res) => {
                    let _ = tx.send(AppMessage::SearchLoaded(res));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Search failed: {}", e)));
                }
            }
        });
    }

    pub fn play_song(&mut self, song: Song) {
        self.player.set_loading(true);
        self.history.push(song.clone());

        let client = self.yt_client.clone();
        let lyrics_engine = self.lyrics_engine.clone();
        let tx = self.tx.clone();
        let s = song.clone();

        self.tokio_handle.spawn(async move {
            // Step 1: Resolve stream URL with matching UA
            let (url, stats, user_agent) = match client.get_stream_url(&s.video_id).await {
                Ok(res) => res,
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to stream: {}", e)));
                    return;
                }
            };

            // Step 2: Fetch lyrics (try LRCLIB first, then YouTube Music)
            let mut lyrics = Vec::new();
            if let Ok(lrc) = lyrics_engine.fetch_synced_lyrics(&s.title, &s.artist, s.duration_seconds).await {
                lyrics = lrc;
            } else if let Ok(Some(plain)) = client.get_lyrics(&s.video_id).await {
                lyrics = LyricsEngine::interpolate_plain_lyrics(&plain, s.duration_seconds);
            }

            // Step 3: Fetch Up Next recommendations
            let up_next = client.get_up_next(&s.video_id).await.unwrap_or_default();

            let _ = tx.send(AppMessage::TrackReady {
                song: s,
                url,
                stats,
                user_agent,
                lyrics,
                up_next,
            });
        });
    }

    fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::HomeLoaded(shelves) => {
                    self.home_shelves = shelves;
                    self.is_loading_home = false;
                }
                AppMessage::SearchLoaded(results) => {
                    self.search_results = results;
                    self.is_searching = false;
                }
                AppMessage::TrackReady {
                    song,
                    url,
                    stats,
                    user_agent,
                    lyrics,
                    up_next,
                } => {
                    if let Err(e) = self.player.play_stream(url, song.clone(), stats, user_agent, 0) {
                        eprintln!("[BitChord] Playback error: {}", e);
                    }
                    self.current_lyrics = lyrics;

                    // Add up_next into queue
                    let mut q = self.queue.lock().unwrap();
                    if q.tracks.is_empty() {
                        let mut all_tracks = vec![song];
                        all_tracks.extend(up_next);
                        q.set_queue(all_tracks, 0);
                    }
                }
                AppMessage::LyricsLoaded(lyrics) => {
                    self.current_lyrics = lyrics;
                }
                AppMessage::Error(err) => {
                    eprintln!("[BitChord Error] {}", err);
                    self.is_searching = false;
                    self.is_loading_home = false;
                    self.player.set_loading(false);
                }
            }
        }
    }
}

impl eframe::App for BitChordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();

        // Request continuous repaint for smooth scrubber and lyrics updates
        ctx.request_repaint_after(std::time::Duration::from_millis(60));

        let current_song = self.player.current_song();
        let stats = self.player.stats();
        let (repeat_mode, shuffle) = {
            let q = self.queue.lock().unwrap();
            (q.repeat_mode, q.shuffle)
        };
        let is_liked = current_song
            .as_ref()
            .map(|s| self.liked_songs.iter().any(|l| l.video_id == s.video_id))
            .unwrap_or(false);

        // 1. Fullscreen Now Playing Overlay (Apple Music Ambient Glow & Synced Lyrics)
        if self.fullscreen_now_playing {
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(BitChordTheme::BG_DEEP))
                .show(ctx, |ui| {
                    let mut on_seek = None;
                    let mut on_close = false;

                    NowPlayingView::show(
                        ui,
                        &self.player,
                        current_song.as_ref(),
                        &stats,
                        &self.current_lyrics,
                        &mut on_seek,
                        &mut on_close,
                    );

                    if let Some(target_ms) = on_seek {
                        self.player.set_elapsed_ms(target_ms);
                    }
                    if on_close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.fullscreen_now_playing = false;
                    }
                });
            return;
        }

        // 2. Navigation Sidebar (Apple Music styled 240px width)
        SidePanel::left("bitchord_sidebar")
            .resizable(false)
            .exact_width(240.0)
            .frame(egui::Frame::none().fill(BitChordTheme::BG_SIDEBAR).inner_margin(Margin::same(0.0)))
            .show(ctx, |ui| {
                let queue_len = self.queue.lock().unwrap().tracks.len();
                let mut trigger_search = false;
                Sidebar::show(
                    ui,
                    &mut self.current_tab,
                    &mut self.search_query,
                    &mut trigger_search,
                    queue_len,
                );
                if trigger_search {
                    self.trigger_search();
                }
            });

        // 3. Bottom Player Bar (Apple Music Floating Glass Pill)
        egui::TopBottomPanel::bottom("bitchord_player_bar")
            .resizable(false)
            .exact_height(78.0)
            .frame(egui::Frame::none().fill(BitChordTheme::BG_MAIN).inner_margin(Margin::symmetric(0.0, 4.0)))
            .show(ctx, |ui| {
                let action = PlayerBar::show(
                    ui,
                    &self.player,
                    current_song.as_ref(),
                    &stats,
                    repeat_mode,
                    shuffle,
                    self.lyrics_open,
                    self.queue_open,
                    is_liked,
                );

                if action.toggle_play {
                    self.player.toggle_play();
                }
                if action.next {
                    let next_song = self.queue.lock().unwrap().next();
                    if let Some(song) = next_song {
                        self.play_song(song);
                    }
                }
                if action.previous {
                    let prev_song = self.queue.lock().unwrap().previous();
                    if let Some(song) = prev_song {
                        self.play_song(song);
                    }
                }
                if action.toggle_shuffle {
                    self.queue.lock().unwrap().toggle_shuffle();
                }
                if action.cycle_repeat {
                    self.queue.lock().unwrap().cycle_repeat();
                }
                if action.toggle_lyrics {
                    self.lyrics_open = !self.lyrics_open;
                }
                if action.toggle_queue {
                    self.queue_open = !self.queue_open;
                }
                if action.toggle_fullscreen {
                    self.fullscreen_now_playing = true;
                }
                if action.toggle_like {
                    if let Some(s) = current_song.as_ref() {
                        if is_liked {
                            self.liked_songs.retain(|l| l.video_id != s.video_id);
                        } else {
                            self.liked_songs.push(s.clone());
                        }
                    }
                }
                if let Some(ms) = action.seek_to_ms {
                    self.player.set_elapsed_ms(ms);
                }
            });

        // 4. Central Content Views
        CentralPanel::default()
            .frame(egui::Frame::none().fill(BitChordTheme::BG_MAIN).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| {
                let mut song_to_play = None;

                match self.current_tab {
                    NavTab::Home => {
                        HomeView::show(ui, &self.home_shelves, self.is_loading_home, &mut song_to_play);
                    }
                    NavTab::Explore => {
                        let mut genre = None;
                        ExploreView::show(ui, &mut genre);
                        if let Some(g) = genre {
                            self.search_query = g;
                            self.current_tab = NavTab::Search;
                            self.trigger_search();
                        }
                    }
                    NavTab::Search => {
                        let mut trigger = false;
                        SearchView::show(
                            ui,
                            &mut self.search_query,
                            &self.search_results,
                            self.is_searching,
                            &mut trigger,
                            &mut song_to_play,
                        );
                        if trigger {
                            self.trigger_search();
                        }
                    }
                    NavTab::Library => {
                        LibraryView::show(ui, &self.liked_songs, &self.history, &mut song_to_play);
                    }
                    NavTab::Settings => {
                        SettingsView::show(ui, &mut self.settings);
                    }
                }

                if let Some(song) = song_to_play {
                    self.play_song(song);
                }
            });
    }
}
