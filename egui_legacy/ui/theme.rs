use egui::{Color32, Stroke};

pub struct BitChordTheme;

impl BitChordTheme {
    // Apple Music dark palette (from tokens.css)
    pub const BG_DEEP: Color32 = Color32::from_rgb(18, 18, 22);       // #121216
    pub const BG_MAIN: Color32 = Color32::from_rgb(26, 26, 31);       // #1a1a1f
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(22, 22, 26);    // #16161a

    pub const SURFACE: Color32 = Color32::from_rgb(34, 34, 42);       // #22222a
    pub const SURFACE_HOVER: Color32 = Color32::from_rgb(46, 46, 56); // #2e2e38
    pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(56, 56, 70);

    // Apple Music Floating Glass Pill (56px height, 28px radius)
    pub const GLASS_PILL: Color32 = Color32::from_rgba_premultiplied(32, 32, 42, 235);
    pub const GLASS_BORDER: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 32);
    pub const GLASS_BORDER_GLOW: Color32 = Color32::from_rgb(0, 210, 255);

    // Accents
    pub const KEY_RED: Color32 = Color32::from_rgb(250, 45, 72);     // Apple Music key red/pink #FA2D48
    pub const KEY_CYAN: Color32 = Color32::from_rgb(0, 210, 255);    // BitChord neon cyan #00D2FF
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 210, 255); // Alias for components
    pub const ACCENT_PINK: Color32 = Color32::from_rgb(250, 45, 72); // Alias for components
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(175, 82, 222);

    // Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(165, 165, 178);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(115, 115, 130);

    // Lyrics Active Glow
    pub const LYRIC_ACTIVE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const LYRIC_INACTIVE: Color32 = Color32::from_rgb(110, 110, 128);

    pub fn glass_stroke() -> Stroke {
        Stroke::new(1.0_f32, Self::GLASS_BORDER)
    }

    pub fn active_stroke() -> Stroke {
        Stroke::new(1.5_f32, Self::KEY_RED)
    }
}
