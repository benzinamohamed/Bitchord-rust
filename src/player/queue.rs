use crate::models::{RepeatMode, Song};

#[derive(Debug, Clone, Default)]
pub struct PlaybackQueue {
    pub tracks: Vec<Song>,
    pub current_index: Option<usize>,
    pub history: Vec<Song>,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            history: Vec::new(),
            repeat_mode: RepeatMode::Off,
            shuffle: false,
        }
    }

    pub fn set_queue(&mut self, tracks: Vec<Song>, start_index: usize) -> Option<Song> {
        if tracks.is_empty() {
            self.tracks.clear();
            self.current_index = None;
            return None;
        }

        self.tracks = tracks;
        let index = start_index.min(self.tracks.len() - 1);
        self.current_index = Some(index);
        let current = self.tracks[index].clone();
        self.history.push(current.clone());
        Some(current)
    }

    pub fn add_track(&mut self, track: Song) {
        self.tracks.push(track);
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    pub fn play_next(&mut self, track: Song) {
        if let Some(idx) = self.current_index {
            self.tracks.insert(idx + 1, track);
        } else {
            self.tracks.push(track);
            self.current_index = Some(0);
        }
    }

    pub fn current(&self) -> Option<&Song> {
        let idx = self.current_index?;
        self.tracks.get(idx)
    }

    pub fn next(&mut self) -> Option<Song> {
        if self.tracks.is_empty() {
            return None;
        }

        let idx = self.current_index.unwrap_or(0);

        if self.repeat_mode == RepeatMode::One {
            return self.tracks.get(idx).cloned();
        }

        let next_idx = idx + 1;
        if next_idx < self.tracks.len() {
            self.current_index = Some(next_idx);
            let song = self.tracks[next_idx].clone();
            self.history.push(song.clone());
            Some(song)
        } else if self.repeat_mode == RepeatMode::All {
            self.current_index = Some(0);
            let song = self.tracks[0].clone();
            self.history.push(song.clone());
            Some(song)
        } else {
            None
        }
    }

    pub fn previous(&mut self) -> Option<Song> {
        if self.tracks.is_empty() {
            return None;
        }

        let idx = self.current_index.unwrap_or(0);
        if idx > 0 {
            let prev_idx = idx - 1;
            self.current_index = Some(prev_idx);
            self.tracks.get(prev_idx).cloned()
        } else if self.repeat_mode == RepeatMode::All {
            let last_idx = self.tracks.len() - 1;
            self.current_index = Some(last_idx);
            self.tracks.get(last_idx).cloned()
        } else {
            self.tracks.get(0).cloned()
        }
    }

    pub fn cycle_repeat(&mut self) -> RepeatMode {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
        self.repeat_mode
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        self.shuffle = !self.shuffle;
        self.shuffle
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
    }
}
