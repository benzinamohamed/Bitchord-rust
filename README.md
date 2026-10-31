# BitChord (Rust Edition for Arch Linux)

An aesthetic, high-performance **YouTube Music desktop client** built with **Rust** and **egui**, tailored for Arch Linux and modern Wayland/X11 desktops.

Inspired directly by [BitChord (Kotlin)](https://github.com/kushagrasinghx/BitChord) and Apple Music's sleek, translucent design philosophy.

---

## ⚡ Features

### 🎧 Audio & Playback
- **Direct YouTube Music Streaming**: Native Opus (160 kbps) and AAC playback via pure Rust audio engine (`rodio` + `symphonia`).
- **Bit-Perfect Lossless Experience**: Low-latency, zero-bloat audio pipeline with hardware acceleration on Arch Linux (`ALSA`, `PulseAudio`, `PipeWire`).
- **Interactive Scrubber**: Seek to any millisecond with smooth dragging and real-time timestamp display.
- **Queue & Autoplay**: Dynamic Up-Next recommendations fetched directly from YouTube Music radio endpoints.
- **Repeat & Shuffle Modes**: Full queue shuffling and repeat (one track or all).

### 🎤 Synchronized Apple Music-Style Lyrics
- **Real-Time Karaoke Lyrics**:
  - Active line glows brightly with smooth font scaling.
  - Inactive lines are softly dimmed for an authentic Apple Music feel.
  - Smooth automatic centering and scrolling following playback timestamps.
  - **Click-to-Seek**: Click on any lyric line to jump directly to that timestamp.
- **Multi-Source Fetching**:
  - Integrated LRCLIB provider with millisecond timestamps.
  - YouTube Music official lyrics integration as fallback.
  - Dynamic interpolation for unsynced tracks so every song has a scrolling lyric view.

### 🎨 Apple Music / BitChord Aesthetics
- **LiquidGlass Frosted UI**: Translucent floating glass player pill, soft borders, and deep graphite tones (`#0B0B0F`, `#101016`).
- **Dynamic Ambient Glow**: Ambient background lighting with cyan and magenta accents.
- **Fullscreen Immersive Now-Playing Screen**:
  - Large album cover with subtle drop shadow.
  - Live synchronized lyrics column.
  - **"Stats for Nerds" badge**: Displays active codec, bitrate (kbps), sample rate (kHz), bit depth, and hi-res audio tags.
- **Categorized Search & Discovery**:
  - **Listen Now**: Recommended shelves, trending playlists, and quick picks.
  - **Explore**: 10 curated musical genres and moods (Pop, Hip-Hop, Electronic, Rock, R&B, Lo-Fi, Indie, Gaming, Classical, Ambient).
  - **Instant Search**: Songs, Artists, and Albums with instant playback.
  - **Library**: Liked Songs and playback history.

---

## 🚀 Installation on Arch Linux

### Method 1: Using the Installer Script
```bash
cd /home/hamoda/bitc/bitchord-rust
./install.sh
```
This installs `bitchord` to `~/.local/bin/bitchord`, creates the desktop icon, and installs the `.desktop` application entry for your application menu (Rofi, Wofi, GNOME, KDE, Hyprland).

### Method 2: Running with Cargo
```bash
cd /home/hamoda/bitc/bitchord-rust
cargo run --release
```

### Method 3: Using PKGBUILD (Arch Linux package)
```bash
cd /home/hamoda/bitc/bitchord-rust
makepkg -si
```

---

## 🛠️ Architecture

```
bitchord-rust/
├── src/
│   ├── main.rs            # Application entry, tokio runtime, and eframe initialization
│   ├── app.rs             # Main application state, background channels & UI orchestration
│   ├── models.rs          # Data types: Song, Album, Artist, Lyrics, AudioQualityStats
│   ├── ytmusic/           # YouTube Music Innertube API client & response parsers
│   ├── player/            # Audio engine with Rodio & symphonia decoding, queue management
│   ├── lyrics/            # Multi-source synced lyrics engine (LRCLIB + YouTube Music)
│   └── ui/                # Apple Music glass widgets, player bar, full-screen now playing
├── assets/
│   └── logo.png           # High-resolution BitChord icon
├── bitchord.desktop       # FreeDesktop standard desktop entry
└── PKGBUILD               # Arch Linux package specification
```
