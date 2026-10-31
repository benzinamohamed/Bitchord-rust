use crate::models::{AudioQualityStats, PlaybackStatus, Song};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Arc<Mutex<Option<Sink>>>,
    status: Arc<Mutex<PlaybackStatus>>,
    volume: Arc<Mutex<f32>>,
    current_song: Arc<Mutex<Option<Song>>>,
    current_stats: Arc<Mutex<AudioQualityStats>>,
    playback_start: Arc<Mutex<Option<Instant>>>,
    elapsed_offset: Arc<Mutex<Duration>>,
    is_loading: Arc<AtomicBool>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("[BitChord] Failed to initialize default audio output: {}", e);
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle,
            sink: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(PlaybackStatus::Stopped)),
            volume: Arc::new(Mutex::new(0.8)),
            current_song: Arc::new(Mutex::new(None)),
            current_stats: Arc::new(Mutex::new(AudioQualityStats::default())),
            playback_start: Arc::new(Mutex::new(None)),
            elapsed_offset: Arc::new(Mutex::new(Duration::ZERO)),
            is_loading: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn play_bytes(&self, bytes: Vec<u8>, song: Song, stats: AudioQualityStats) -> Result<(), String> {
        let handle = match &self.stream_handle {
            Some(h) => h,
            None => return Err("Audio output stream handle is not available".to_string()),
        };

        let cursor = Cursor::new(bytes);
        let decoder = match Decoder::new(cursor) {
            Ok(d) => d,
            Err(e) => return Err(format!("Failed to decode audio format: {}", e)),
        };

        // Create new sink
        let new_sink = match Sink::try_new(handle) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to create sink: {}", e)),
        };

        let vol = *self.volume.lock().unwrap();
        new_sink.set_volume(vol);
        new_sink.append(decoder);
        new_sink.play();

        // Update active state
        {
            let mut sink_lock = self.sink.lock().unwrap();
            *sink_lock = Some(new_sink);
        }
        {
            let mut song_lock = self.current_song.lock().unwrap();
            *song_lock = Some(song);
        }
        {
            let mut stats_lock = self.current_stats.lock().unwrap();
            *stats_lock = stats;
        }
        {
            let mut status_lock = self.status.lock().unwrap();
            *status_lock = PlaybackStatus::Playing;
        }
        {
            let mut start_lock = self.playback_start.lock().unwrap();
            *start_lock = Some(Instant::now());
        }
        {
            let mut offset_lock = self.elapsed_offset.lock().unwrap();
            *offset_lock = Duration::ZERO;
        }

        self.is_loading.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn toggle_play(&self) {
        let mut status = self.status.lock().unwrap();
        if let Some(sink) = self.sink.lock().unwrap().as_ref() {
            match *status {
                PlaybackStatus::Playing => {
                    sink.pause();
                    *status = PlaybackStatus::Paused;
                    if let Some(start) = self.playback_start.lock().unwrap().take() {
                        let mut offset = self.elapsed_offset.lock().unwrap();
                        *offset += start.elapsed();
                    }
                }
                PlaybackStatus::Paused => {
                    sink.play();
                    *status = PlaybackStatus::Playing;
                    let mut start = self.playback_start.lock().unwrap();
                    *start = Some(Instant::now());
                }
                _ => {}
            }
        }
    }

    pub fn pause(&self) {
        let mut status = self.status.lock().unwrap();
        if *status == PlaybackStatus::Playing {
            if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                sink.pause();
                *status = PlaybackStatus::Paused;
                if let Some(start) = self.playback_start.lock().unwrap().take() {
                    let mut offset = self.elapsed_offset.lock().unwrap();
                    *offset += start.elapsed();
                }
            }
        }
    }

    pub fn resume(&self) {
        let mut status = self.status.lock().unwrap();
        if *status == PlaybackStatus::Paused {
            if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                sink.play();
                *status = PlaybackStatus::Playing;
                let mut start = self.playback_start.lock().unwrap();
                *start = Some(Instant::now());
            }
        }
    }

    pub fn stop(&self) {
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
        *self.status.lock().unwrap() = PlaybackStatus::Stopped;
        *self.playback_start.lock().unwrap() = None;
        *self.elapsed_offset.lock().unwrap() = Duration::ZERO;
    }

    pub fn set_volume(&self, vol: f32) {
        let clamped = vol.clamp(0.0, 1.0);
        *self.volume.lock().unwrap() = clamped;
        if let Some(sink) = self.sink.lock().unwrap().as_ref() {
            sink.set_volume(clamped);
        }
    }

    pub fn volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    pub fn status(&self) -> PlaybackStatus {
        // If playing and sink finished, mark stopped
        let mut status = self.status.lock().unwrap();
        if *status == PlaybackStatus::Playing {
            if let Some(sink) = self.sink.lock().unwrap().as_ref() {
                if sink.empty() {
                    *status = PlaybackStatus::Stopped;
                }
            }
        }
        *status
    }

    pub fn current_song(&self) -> Option<Song> {
        self.current_song.lock().unwrap().clone()
    }

    pub fn stats(&self) -> AudioQualityStats {
        self.current_stats.lock().unwrap().clone()
    }

    pub fn elapsed(&self) -> Duration {
        let base_offset = *self.elapsed_offset.lock().unwrap();
        if *self.status.lock().unwrap() == PlaybackStatus::Playing {
            if let Some(start) = *self.playback_start.lock().unwrap() {
                return base_offset + start.elapsed();
            }
        }
        base_offset
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    pub fn set_elapsed_ms(&self, ms: u64) {
        let mut offset = self.elapsed_offset.lock().unwrap();
        *offset = Duration::from_millis(ms);
        let mut start = self.playback_start.lock().unwrap();
        if *self.status.lock().unwrap() == PlaybackStatus::Playing {
            *start = Some(Instant::now());
        }
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading.load(Ordering::SeqCst)
    }

    pub fn set_loading(&self, loading: bool) {
        self.is_loading.store(loading, Ordering::SeqCst);
        if loading {
            *self.status.lock().unwrap() = PlaybackStatus::Buffering;
        }
    }
}
