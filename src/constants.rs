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

// Font size specific to cyberpunk-styled text
pub const CYBERPUNK_FONT_SIZE: f32 = 24.0; // Font size for UI text (song selection, buttons, etc.)

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
