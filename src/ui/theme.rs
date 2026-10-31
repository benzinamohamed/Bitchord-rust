use egui::{Color32, Stroke};

pub struct BitChordTheme;

impl BitChordTheme {
    // Apple Music dark palette
    pub const BG_DEEP: Color32 = Color32::from_rgb(11, 11, 15);
    pub const BG_MAIN: Color32 = Color32::from_rgb(16, 16, 22);
    pub const SURFACE: Color32 = Color32::from_rgb(24, 24, 32);
    pub const SURFACE_HOVER: Color32 = Color32::from_rgb(34, 34, 46);
    pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(42, 42, 58);

    // Floating Glass Pill
    pub const GLASS_PILL: Color32 = Color32::from_rgb(22, 22, 30);
    pub const GLASS_BORDER: Color32 = Color32::from_rgb(50, 50, 65);
    pub const GLASS_BORDER_GLOW: Color32 = Color32::from_rgb(0, 150, 180);

    // Apple / BitChord Accents
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 210, 255);
    pub const ACCENT_PINK: Color32 = Color32::from_rgb(250, 45, 72);
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(175, 82, 222);

    // Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 245, 250);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 175);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 100, 115);

    // Lyrics Active Glow
    pub const LYRIC_ACTIVE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const LYRIC_INACTIVE: Color32 = Color32::from_rgb(105, 105, 120);

    pub fn glass_stroke() -> Stroke {
        Stroke::new(1.0, Self::GLASS_BORDER)
    }

    pub fn active_stroke() -> Stroke {
        Stroke::new(1.5, Self::ACCENT_CYAN)
    }
}
