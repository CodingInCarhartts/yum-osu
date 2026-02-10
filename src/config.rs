// src/config.rs

use macroquad::input::KeyCode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Game configuration settings for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Key bindings for gameplay
    pub key_bindings: KeyBindings,
    /// Visual theme settings
    pub theme: ThemeConfig,
    /// Audio settings
    pub audio: AudioConfig,
    /// Practice mode settings
    pub practice: PracticeConfig,
    /// Whether to save analytics
    pub save_analytics: bool,
}

/// Key bindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    /// Primary hit key
    pub primary_hit: String,
    /// Secondary hit key  
    pub secondary_hit: String,
    /// Pause key
    pub pause: String,
    /// Exit key
    pub exit: String,
    /// Navigate up
    pub navigate_up: String,
    /// Navigate down
    pub navigate_down: String,
    /// Select/confirm
    pub select: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            primary_hit: "KeyA".to_string(),
            secondary_hit: "KeyS".to_string(),
            pause: "Escape".to_string(),
            exit: "Escape".to_string(),
            navigate_up: "Up".to_string(),
            navigate_down: "Down".to_string(),
            select: "Enter".to_string(),
        }
    }
}

impl KeyBindings {
    /// Get the primary hit key as KeyCode
    pub fn primary_hit_key(&self) -> KeyCode {
        string_to_keycode(&self.primary_hit)
    }

    /// Get the secondary hit key as KeyCode
    pub fn secondary_hit_key(&self) -> KeyCode {
        string_to_keycode(&self.secondary_hit)
    }

    /// Get the pause key as KeyCode
    pub fn pause_key(&self) -> KeyCode {
        string_to_keycode(&self.pause)
    }

    /// Get the exit key as KeyCode
    pub fn exit_key(&self) -> KeyCode {
        string_to_keycode(&self.exit)
    }

    /// Get the navigate up key as KeyCode
    pub fn navigate_up_key(&self) -> KeyCode {
        string_to_keycode(&self.navigate_up)
    }

    /// Get the navigate down key as KeyCode
    pub fn navigate_down_key(&self) -> KeyCode {
        string_to_keycode(&self.navigate_down)
    }

    /// Get the select key as KeyCode
    pub fn select_key(&self) -> KeyCode {
        string_to_keycode(&self.select)
    }
}

/// Convert a string to a KeyCode
fn string_to_keycode(s: &str) -> KeyCode {
    match s {
        "KeyA" => KeyCode::A,
        "KeyB" => KeyCode::B,
        "KeyC" => KeyCode::C,
        "KeyD" => KeyCode::D,
        "KeyE" => KeyCode::E,
        "KeyF" => KeyCode::F,
        "KeyG" => KeyCode::G,
        "KeyH" => KeyCode::H,
        "KeyI" => KeyCode::I,
        "KeyJ" => KeyCode::J,
        "KeyK" => KeyCode::K,
        "KeyL" => KeyCode::L,
        "KeyM" => KeyCode::M,
        "KeyN" => KeyCode::N,
        "KeyO" => KeyCode::O,
        "KeyP" => KeyCode::P,
        "KeyQ" => KeyCode::Q,
        "KeyR" => KeyCode::R,
        "KeyS" => KeyCode::S,
        "KeyT" => KeyCode::T,
        "KeyU" => KeyCode::U,
        "KeyV" => KeyCode::V,
        "KeyW" => KeyCode::W,
        "KeyX" => KeyCode::X,
        "KeyY" => KeyCode::Y,
        "KeyZ" => KeyCode::Z,
        "Space" => KeyCode::Space,
        "Enter" => KeyCode::Enter,
        "Escape" => KeyCode::Escape,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Tab" => KeyCode::Tab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Slash" => KeyCode::Slash,
        "Backslash" => KeyCode::Backslash,
        "Comma" => KeyCode::Comma,
        "Period" => KeyCode::Period,
        "Semicolon" => KeyCode::Semicolon,
        "Apostrophe" => KeyCode::Apostrophe,
        "Minus" => KeyCode::Minus,
        "Equal" => KeyCode::Equal,
        "LeftBracket" => KeyCode::LeftBracket,
        "RightBracket" => KeyCode::RightBracket,
        "Grave" => KeyCode::Grave,
        "Key0" => KeyCode::Key0,
        "Key1" => KeyCode::Key1,
        "Key2" => KeyCode::Key2,
        "Key3" => KeyCode::Key3,
        "Key4" => KeyCode::Key4,
        "Key5" => KeyCode::Key5,
        "Key6" => KeyCode::Key6,
        "Key7" => KeyCode::Key7,
        "Key8" => KeyCode::Key8,
        "Key9" => KeyCode::Key9,
        "Kp0" => KeyCode::Kp0,
        "Kp1" => KeyCode::Kp1,
        "Kp2" => KeyCode::Kp2,
        "Kp3" => KeyCode::Kp3,
        "Kp4" => KeyCode::Kp4,
        "Kp5" => KeyCode::Kp5,
        "Kp6" => KeyCode::Kp6,
        "Kp7" => KeyCode::Kp7,
        "Kp8" => KeyCode::Kp8,
        "Kp9" => KeyCode::Kp9,
        "KpDecimal" => KeyCode::KpDecimal,
        "KpDivide" => KeyCode::KpDivide,
        "KpMultiply" => KeyCode::KpMultiply,
        "KpSubtract" => KeyCode::KpSubtract,
        "KpAdd" => KeyCode::KpAdd,
        "KpEnter" => KeyCode::KpEnter,
        "KpEqual" => KeyCode::KpEqual,
        "LeftShift" => KeyCode::LeftShift,
        "LeftControl" => KeyCode::LeftControl,
        "LeftAlt" => KeyCode::LeftAlt,
        "LeftSuper" => KeyCode::LeftSuper,
        "RightShift" => KeyCode::RightShift,
        "RightControl" => KeyCode::RightControl,
        "RightAlt" => KeyCode::RightAlt,
        "RightSuper" => KeyCode::RightSuper,
        _ => KeyCode::A,
    }
}

/// Get all available keys for binding
pub fn get_available_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("KeyA", "A"),
        ("KeyB", "B"),
        ("KeyC", "C"),
        ("KeyD", "D"),
        ("KeyE", "E"),
        ("KeyF", "F"),
        ("KeyG", "G"),
        ("KeyH", "H"),
        ("KeyI", "I"),
        ("KeyJ", "J"),
        ("KeyK", "K"),
        ("KeyL", "L"),
        ("KeyM", "M"),
        ("KeyN", "N"),
        ("KeyO", "O"),
        ("KeyP", "P"),
        ("KeyQ", "Q"),
        ("KeyR", "R"),
        ("KeyS", "S"),
        ("KeyT", "T"),
        ("KeyU", "U"),
        ("KeyV", "V"),
        ("KeyW", "W"),
        ("KeyX", "X"),
        ("KeyY", "Y"),
        ("KeyZ", "Z"),
        ("Space", "Space"),
        ("Enter", "Enter"),
        ("Escape", "Escape"),
        ("Tab", "Tab"),
        ("Backspace", "Backspace"),
        ("Delete", "Delete"),
        ("Up", "Up Arrow"),
        ("Down", "Down Arrow"),
        ("Left", "Left Arrow"),
        ("Right", "Right Arrow"),
        ("Slash", "/"),
        ("Comma", ","),
        ("Period", "."),
        ("Semicolon", ";"),
        ("Apostrophe", "'"),
        ("Minus", "-"),
        ("Equal", "="),
        ("LeftBracket", "["),
        ("RightBracket", "]"),
        ("Grave", "`"),
        ("Key0", "0"),
        ("Key1", "1"),
        ("Key2", "2"),
        ("Key3", "3"),
        ("Key4", "4"),
        ("Key5", "5"),
        ("Key6", "6"),
        ("Key7", "7"),
        ("Key8", "8"),
        ("Key9", "9"),
        ("Kp0", "Numpad 0"),
        ("Kp1", "Numpad 1"),
        ("Kp2", "Numpad 2"),
        ("Kp3", "Numpad 3"),
        ("Kp4", "Numpad 4"),
        ("Kp5", "Numpad 5"),
        ("Kp6", "Numpad 6"),
        ("Kp7", "Numpad 7"),
        ("Kp8", "Numpad 8"),
        ("Kp9", "Numpad 9"),
    ]
}

/// Visual theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Primary accent color (hex string)
    pub primary_color: String,
    /// Secondary accent color (hex string)
    pub secondary_color: String,
    /// Circle color (hex string)
    pub circle_color: String,
    /// Background style
    pub background_style: BackgroundStyle,
    /// Circle size multiplier
    pub circle_size: f32,
    /// Enable particle effects
    pub particles_enabled: bool,
    /// Enable screen shake on hit
    pub screen_shake: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: "#FF12B8".to_string(),   // Neon pink
            secondary_color: "#00BFFF".to_string(), // Neon blue
            circle_color: "#00BFFF".to_string(),    // Neon blue
            background_style: BackgroundStyle::Cyberpunk,
            circle_size: 1.0,
            particles_enabled: true,
            screen_shake: true,
        }
    }
}

/// Background style options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundStyle {
    Cyberpunk,
    Dark,
    Minimal,
    Gradient,
}

impl BackgroundStyle {
    /// Get all available background styles
    pub fn all() -> Vec<(BackgroundStyle, &'static str)> {
        vec![
            (BackgroundStyle::Cyberpunk, "Cyberpunk"),
            (BackgroundStyle::Dark, "Dark"),
            (BackgroundStyle::Minimal, "Minimal"),
            (BackgroundStyle::Gradient, "Gradient"),
        ]
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
    /// Music volume (0.0 - 1.0)
    pub music_volume: f32,
    /// Effects volume (0.0 - 1.0)
    pub effects_volume: f32,
    /// Enable audio visualization
    pub visualizer_enabled: bool,
    /// Audio buffer size
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            effects_volume: 1.0,
            visualizer_enabled: true,
            buffer_size: 1024,
        }
    }
}

/// Practice mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeConfig {
    /// Playback speed multiplier (0.25 - 2.0)
    pub playback_speed: f32,
    /// Enable no-fail mode
    pub no_fail: bool,
    /// Enable autoplay
    pub autoplay: bool,
    /// Enable hit sounds
    pub hit_sounds: bool,
    /// Loop section start time (in seconds, None if not looping)
    pub loop_start: Option<f64>,
    /// Loop section end time (in seconds, None if not looping)
    pub loop_end: Option<f64>,
}

impl Default for PracticeConfig {
    fn default() -> Self {
        Self {
            playback_speed: 1.0,
            no_fail: false,
            autoplay: false,
            hit_sounds: true,
            loop_start: None,
            loop_end: None,
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            key_bindings: KeyBindings::default(),
            theme: ThemeConfig::default(),
            audio: AudioConfig::default(),
            practice: PracticeConfig::default(),
            save_analytics: true,
        }
    }
}

impl GameConfig {
    /// Load configuration from file or create default
    pub fn load() -> Self {
        let config_path = "config.json";
        if Path::new(config_path).exists() {
            match fs::read_to_string(config_path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Failed to parse config: {}, using default", e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read config: {}, using default", e);
                    Self::default()
                }
            }
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    /// Save configuration to file
    pub fn save(&self) {
        let config_path = "config.json";
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(config_path, json) {
                    eprintln!("Failed to save config: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
            }
        }
    }

    /// Reset to default configuration
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
        self.save();
    }
}

/// Settings menu state
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Current settings tab
    pub current_tab: SettingsTab,
    /// Whether we're waiting for a key input
    pub waiting_for_key: Option<KeyBindingType>,
    /// Selected item index for keyboard navigation
    pub selected_index: usize,
    /// Scroll position for settings menu
    pub scroll_y: f32,
}

impl SettingsState {
    /// Create a new settings state
    pub fn new() -> Self {
        Self {
            current_tab: SettingsTab::General,
            waiting_for_key: None,
            selected_index: 0,
            scroll_y: 0.0,
        }
    }
}

/// Settings tabs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    KeyBindings,
    Theme,
    Audio,
    Practice,
}

impl SettingsTab {
    /// Get all settings tabs
    pub fn all() -> Vec<(SettingsTab, &'static str)> {
        vec![
            (SettingsTab::General, "General"),
            (SettingsTab::KeyBindings, "Key Bindings"),
            (SettingsTab::Theme, "Theme"),
            (SettingsTab::Audio, "Audio"),
            (SettingsTab::Practice, "Practice"),
        ]
    }

    /// Get the next tab
    pub fn next(&self) -> SettingsTab {
        match self {
            SettingsTab::General => SettingsTab::KeyBindings,
            SettingsTab::KeyBindings => SettingsTab::Theme,
            SettingsTab::Theme => SettingsTab::Audio,
            SettingsTab::Audio => SettingsTab::Practice,
            SettingsTab::Practice => SettingsTab::General,
        }
    }

    /// Get the previous tab
    pub fn previous(&self) -> SettingsTab {
        match self {
            SettingsTab::General => SettingsTab::Practice,
            SettingsTab::KeyBindings => SettingsTab::General,
            SettingsTab::Theme => SettingsTab::KeyBindings,
            SettingsTab::Audio => SettingsTab::Theme,
            SettingsTab::Practice => SettingsTab::Audio,
        }
    }
}

/// Types of key bindings that can be customized
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyBindingType {
    PrimaryHit,
    SecondaryHit,
    Pause,
    NavigateUp,
    NavigateDown,
    Select,
}

impl KeyBindingType {
    /// Get display name for the key binding type
    pub fn display_name(&self) -> &'static str {
        match self {
            KeyBindingType::PrimaryHit => "Primary Hit",
            KeyBindingType::SecondaryHit => "Secondary Hit",
            KeyBindingType::Pause => "Pause",
            KeyBindingType::NavigateUp => "Navigate Up",
            KeyBindingType::NavigateDown => "Navigate Down",
            KeyBindingType::Select => "Select / Confirm",
        }
    }

    /// Get all key binding types
    pub fn all() -> Vec<KeyBindingType> {
        vec![
            KeyBindingType::PrimaryHit,
            KeyBindingType::SecondaryHit,
            KeyBindingType::Pause,
            KeyBindingType::NavigateUp,
            KeyBindingType::NavigateDown,
            KeyBindingType::Select,
        ]
    }
}
