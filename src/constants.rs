// src/constants.rs

use macroquad::prelude::*;

// Timing and shrink behavior
pub const SHRINK_TIME: f64 = 1.5; // Time it takes for a circle to shrink
pub const CIRCLE_MAX_RADIUS: f32 = 100.0; // Maximum radius of circles
pub const OUTLINE_THICKNESS: f32 = 2.0; // Thickness of the circle outline

// Score display styling
pub const SCORE_FONT_SIZE: f32 = 40.0; // Size of the score font

// Outlines and backgrounds
pub const OUTLINE_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.5); // Semi-transparent black outline
pub const DARK_BACKGROUND: Color = Color::new(0.05, 0.05, 0.1, 1.0); // Dark background to enhance neon colors

// Positioning for score display
pub const DRAW_SCORE_X: f32 = 20.0; // X position for score
pub const DRAW_SCORE_Y: f32 = 40.0; // Y position for score

// Song selection and entry heights
pub const SONG_ENTRY_HEIGHT: f32 = 40.0; // Height of each song entry
pub const FONT_SIZE: u16 = 30; // General font size for text

// Countdown behavior
pub const COUNTDOWN_DURATION: f64 = 5.0; // Countdown before game starts

// Cyberpunk neon colors
pub const NEON_PINK: Color = Color::new(1.0, 0.07, 0.58, 1.0); // Neon pink for active UI elements
pub const NEON_BLUE: Color = Color::new(0.0, 0.75, 1.0, 1.0); // Neon blue for circles and background highlights
pub const NEON_PURPLE: Color = Color::new(0.6, 0.0, 1.0, 1.0); // Neon purple for outlines and accents
pub const NEON_GREEN: Color = Color::new(0.0, 1.0, 0.5, 1.0); // Neon green for success or active states
pub const NEON_ORANGE: Color = Color::new(1.0, 0.5, 0.0, 1.0); // Neon orange for errors
pub const NEON_CYAN: Color = Color::new(0.0, 1.0, 1.0, 1.0); // Neon cyan for highlights
pub const NEON_YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0); // Neon yellow for warnings and info
pub const NEON_RED: Color = Color::new(1.0, 0.0, 0.0, 1.0); // Neon red for critical errors

// Standard colors
pub const GRAY: Color = Color::new(0.5, 0.5, 0.5, 1.0); // Gray for inactive elements
pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0); // White for text
pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0); // Black for backgrounds

// Font size specific to cyberpunk-styled text
pub const CYBERPUNK_FONT_SIZE: f32 = 24.0; // Font size for UI text (song selection, buttons, etc.)

// UI Constants
pub const BUTTON_WIDTH: f32 = 250.0;
pub const BUTTON_HEIGHT: f32 = 50.0;
pub const BUTTON_SPACING: f32 = 20.0;
pub const TAB_HEIGHT: f32 = 40.0;
pub const SLIDER_WIDTH: f32 = 200.0;
pub const SLIDER_HEIGHT: f32 = 10.0;

// Analytics colors
pub const ACCENT_COLOR: Color = NEON_CYAN;
pub const SUCCESS_COLOR: Color = NEON_GREEN;
pub const WARNING_COLOR: Color = NEON_YELLOW;
pub const ERROR_COLOR: Color = NEON_ORANGE;

// Grade colors
pub const GRADE_SS_COLOR: Color = Color::new(1.0, 0.84, 0.0, 1.0);
pub const GRADE_S_COLOR: Color = Color::new(1.0, 0.5, 0.0, 1.0);
pub const GRADE_A_COLOR: Color = NEON_GREEN;
pub const GRADE_B_COLOR: Color = NEON_BLUE;
pub const GRADE_C_COLOR: Color = NEON_PURPLE;
pub const GRADE_D_COLOR: Color = NEON_PINK;
pub const GRADE_F_COLOR: Color = Color::new(1.0, 0.0, 0.0, 1.0);

// Practice mode constants
pub const MIN_PLAYBACK_SPEED: f32 = 0.25;
pub const MAX_PLAYBACK_SPEED: f32 = 2.0;
pub const SPEED_STEP: f32 = 0.25;

// Combo milestones for visual effects
pub const COMBO_MILESTONES: [u32; 5] = [10, 25, 50, 100, 200];

// Animation constants
pub const PULSE_SPEED: f32 = 2.0;
pub const GLOW_INTENSITY: f32 = 0.5;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "YumOsu!".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        fullscreen: false,
        high_dpi: true,
        platform: miniquad::conf::Platform {
            framebuffer_alpha: false,
            swap_interval: Some(1), // Enable VSync
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Get grade color based on grade string
pub fn get_grade_color(grade: &str) -> Color {
    match grade {
        "SS" => GRADE_SS_COLOR,
        "S" => GRADE_S_COLOR,
        "A" => GRADE_A_COLOR,
        "B" => GRADE_B_COLOR,
        "C" => GRADE_C_COLOR,
        "D" => GRADE_D_COLOR,
        _ => GRADE_F_COLOR,
    }
}

/// Hex string to Color conversion
pub fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

    Some(Color::new(r, g, b, 1.0))
}

/// Color to hex string conversion
pub fn color_to_hex(color: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    )
}
