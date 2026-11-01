use crate::models::{AudioQualityStats, PlaybackStatus, Song};
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct AudioPlayer {
    stream_handle: Option<OutputStreamHandle>,
    sink: Arc<Mutex<Option<Arc<Sink>>>>,
    active_process: Arc<Mutex<Option<Child>>>,
    stream_abort: Arc<AtomicBool>,
    status: Arc<Mutex<PlaybackStatus>>,
    volume: Arc<Mutex<f32>>,
    current_song: Arc<Mutex<Option<Song>>>,
    current_stats: Arc<Mutex<AudioQualityStats>>,
    playback_start: Arc<Mutex<Option<Instant>>>,
    elapsed_offset: Arc<Mutex<Duration>>,
    current_stream_url: Arc<Mutex<Option<String>>>,
    current_user_agent: Arc<Mutex<Option<String>>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let stream_handle = match OutputStream::try_default() {
            Ok((s, h)) => {
                // Keep output stream alive for the entire process lifetime
                Box::leak(Box::new(s));
                Some(h)
            }
            Err(e) => {
                eprintln!("[BitChord] Audio device warning: {}", e);
                None
            }
        };

        Self {
            stream_handle,
            sink: Arc::new(Mutex::new(None)),
            active_process: Arc::new(Mutex::new(None)),
            stream_abort: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new(PlaybackStatus::Stopped)),
            volume: Arc::new(Mutex::new(0.8)),
            current_song: Arc::new(Mutex::new(None)),
            current_stats: Arc::new(Mutex::new(AudioQualityStats::default())),
            playback_start: Arc::new(Mutex::new(None)),
            elapsed_offset: Arc::new(Mutex::new(Duration::ZERO)),
            current_stream_url: Arc::new(Mutex::new(None)),
            current_user_agent: Arc::new(Mutex::new(None)),
        }
    }

    pub fn play_stream(
        &self,
        url: String,
        song: Song,
        stats: AudioQualityStats,
        user_agent: String,
        start_sec: u64,
    ) -> Result<(), String> {
        let handle = match &self.stream_handle {
            Some(h) => h.clone(),
            None => return Err("Audio output handle unavailable".to_string()),
        };

        // Stop current playback and terminate previous ffmpeg stream
        self.stop();

        *self.current_stream_url.lock().unwrap() = Some(url.clone());
        *self.current_user_agent.lock().unwrap() = Some(user_agent.clone());
        *self.current_song.lock().unwrap() = Some(song);
        *self.current_stats.lock().unwrap() = stats;
        *self.status.lock().unwrap() = PlaybackStatus::Buffering;
        *self.elapsed_offset.lock().unwrap() = Duration::from_secs(start_sec);

        let new_sink = match Sink::try_new(&handle) {
            Ok(s) => Arc::new(s),
            Err(e) => return Err(format!("Failed to create sink: {}", e)),
        };

        let vol = *self.volume.lock().unwrap();
        new_sink.set_volume(vol);

        *self.sink.lock().unwrap() = Some(new_sink.clone());

        // Spawn FFmpeg to stream and decode directly with exact matching User-Agent
        let mut cmd = Command::new("ffmpeg");
        if start_sec > 0 {
            cmd.args(["-ss", &start_sec.to_string()]);
        }
        let header_arg = format!("User-Agent: {}\r\n", user_agent);
        cmd.args([
            "-reconnect",
            "1",
            "-reconnect_streamed",
            "1",
            "-reconnect_delay_max",
            "5",
            "-headers",
            &header_arg,
            "-i",
            &url,
            "-f",
            "s16le",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to spawn ffmpeg: {}", e)),
        };

        let mut stdout = match child.stdout.take() {
            Some(s) => s,
            None => return Err("Failed to capture ffmpeg stdout".to_string()),
        };

        *self.active_process.lock().unwrap() = Some(child);
        self.stream_abort.store(false, Ordering::SeqCst);

        let abort_flag = self.stream_abort.clone();
        let status_arc = self.status.clone();
        let start_arc = self.playback_start.clone();

        std::thread::spawn(move || {
            let mut buf = [0u8; 48000 * 2]; // 0.5s chunks of 16-bit 48kHz stereo
            let mut first_chunk = true;

            while !abort_flag.load(Ordering::SeqCst) {
                match stdout.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let samples: Vec<i16> = buf[..n]
                            .chunks_exact(2)
                            .map(|c| i16::from_le_bytes([c[0], c[1]]))
                            .collect();

                        if !samples.is_empty() {
                            let source = SamplesBuffer::new(2, 48000, samples);
                            new_sink.append(source);

                            if first_chunk {
                                *status_arc.lock().unwrap() = PlaybackStatus::Playing;
                                *start_arc.lock().unwrap() = Some(Instant::now());
                                first_chunk = false;
                            }
                        }

                        // Sleep briefly if buffer has plenty of samples queued
                        while new_sink.len() > 6 && !abort_flag.load(Ordering::SeqCst) {
                            std::thread::sleep(Duration::from_millis(150));
                        }
                    }
                    Err(_) => break,
                }
            }

            if !abort_flag.load(Ordering::SeqCst) {
                new_sink.sleep_until_end();
                *status_arc.lock().unwrap() = PlaybackStatus::Stopped;
            }
        });

        Ok(())
    }

    pub fn set_loading(&self, loading: bool) {
        let mut status = self.status.lock().unwrap();
        if loading {
            *status = PlaybackStatus::Buffering;
        } else if *status == PlaybackStatus::Buffering {
            *status = PlaybackStatus::Stopped;
        }
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

    pub fn stop(&self) {
        self.stream_abort.store(true, Ordering::SeqCst);
        if let Some(mut child) = self.active_process.lock().unwrap().take() {
            let _ = child.kill();
        }
        if let Some(sink) = self.sink.lock().unwrap().take() {
            sink.stop();
        }
        *self.status.lock().unwrap() = PlaybackStatus::Stopped;
        *self.playback_start.lock().unwrap() = None;
        *self.elapsed_offset.lock().unwrap() = Duration::ZERO;
    }

    pub fn seek_to_sec(&self, sec: u64) {
        if let (Some(url), Some(ua), Some(song), stats) = (
            self.current_stream_url.lock().unwrap().clone(),
            self.current_user_agent.lock().unwrap().clone(),
            self.current_song(),
            self.stats(),
        ) {
            let _ = self.play_stream(url, song, stats, ua, sec);
        }
    }

    pub fn set_elapsed_ms(&self, ms: u64) {
        self.seek_to_sec(ms / 1000);
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
        *self.status.lock().unwrap()
    }

    pub fn current_song(&self) -> Option<Song> {
        self.current_song.lock().unwrap().clone()
    }

    pub fn stats(&self) -> AudioQualityStats {
        self.current_stats.lock().unwrap().clone()
    }

    pub fn elapsed(&self) -> Duration {
        let offset = *self.elapsed_offset.lock().unwrap();
        if *self.status.lock().unwrap() == PlaybackStatus::Playing {
            if let Some(start) = *self.playback_start.lock().unwrap() {
                return offset + start.elapsed();
            }
        }
        offset
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    pub fn elapsed_sec(&self) -> u64 {
        self.elapsed().as_secs()
    }
}
