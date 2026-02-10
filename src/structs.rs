// src/structs.rs

use bevy::prelude::*;
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::time::Instant;

use crate::analytics::ActiveSession;
use crate::config::GameConfig;

/// UI Assets container
#[derive(Resource, Clone)]
pub struct GameAssets {
    pub cyberpunk_font: Handle<Font>,
}

/// Song selection state
#[derive(Debug, Clone, Resource)]
pub struct SongSelectionState {
    pub scroll_pos: f32,
    pub selected_song: Option<String>,
    /// Whether practice mode is enabled
    pub practice_mode: bool,
    /// Selected playback speed for practice mode
    pub playback_speed: f32,
}

impl Default for SongSelectionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SongSelectionState {
    /// Create new song selection state
    pub fn new() -> Self {
        Self {
            scroll_pos: 0.0,
            selected_song: None,
            practice_mode: false,
            playback_speed: 1.0,
        }
    }
}

/// Main game state enum
#[derive(Debug, Clone)]
pub enum GameState {
    Menu,
    SongSelection,
    /// Practice tools menu
    PracticeMenu,
    Playing,
    Settings,
    Analytics,
    Exit,
    Loading {
        rx: mpsc::Receiver<Vec<f64>>,
        start_time: Instant,
    },
    ReadyToPlay {
        beats: Vec<f64>,
        ready_time: Instant,
        source: Option<Decoder<BufReader<File>>>,
    },
    Visualizing(Box<VisualizingState>),
    End(Box<EndState>),
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Menu
    }
}

/// Game circle structure
#[derive(Debug, Clone)]
pub struct GameCircle {
    pub position: Vec2,
    pub spawn_time: f64,
    pub hit_time: f64,
    pub max_radius: f32,
    pub hit: bool,
    pub missed: bool,
}

/// Floating text for feedback
#[derive(Debug, Clone)]
pub struct FloatingText {
    pub text: String,
    pub position: Vec2,
    pub spawn_time: f64,
    pub duration: f64,
    /// Text color
    pub color: (f32, f32, f32),
}

/// Visualizing/gameplay state
#[derive(Debug, Clone)]
pub struct VisualizingState {
    pub beats: Vec<f64>,
    pub start_time: Instant,
    pub circles: Vec<GameCircle>,
    pub score: i32,
    pub floating_texts: Vec<FloatingText>,
    /// Current game configuration
    pub config: GameConfig,
    /// Active analytics session
    pub active_session: Option<ActiveSession>,
    /// Whether practice mode is active
    pub practice_mode: bool,
    /// Playback speed (1.0 = normal)
    pub playback_speed: f32,
    /// No-fail mode enabled
    pub no_fail: bool,
    /// Song name
    pub song_name: String,
    /// Combo counter
    pub combo: u32,
    /// Max combo achieved
    pub max_combo: u32,
}

impl VisualizingState {
    /// Create new visualizing state
    pub fn new(
        beats: Vec<f64>,
        circles: Vec<GameCircle>,
        config: GameConfig,
        song_name: String,
    ) -> Self {
        let practice_mode = config.practice.autoplay || config.practice.no_fail;
        let playback_speed = config.practice.playback_speed;
        let no_fail = config.practice.no_fail;

        let active_session = if config.save_analytics {
            Some(ActiveSession::new(
                song_name.clone(),
                practice_mode,
                playback_speed,
            ))
        } else {
            None
        };

        Self {
            beats,
            start_time: Instant::now(),
            circles,
            score: 0,
            floating_texts: Vec::new(),
            config,
            active_session,
            practice_mode,
            playback_speed,
            no_fail,
            song_name,
            combo: 0,
            max_combo: 0,
        }
    }

    /// Record a hit with timing
    pub fn record_hit(&mut self, points: i32, timing_ms: f32) {
        self.score += points;

        // Update combo
        if points > 0 {
            self.combo += 1;
            if self.combo > self.max_combo {
                self.max_combo = self.combo;
            }
        } else {
            self.combo = 0;
        }

        // Record in analytics session
        if let Some(ref mut session) = self.active_session {
            session.record_hit(points, timing_ms);
        }
    }

    /// Record a miss
    pub fn record_miss(&mut self) {
        self.combo = 0;

        if let Some(ref mut session) = self.active_session {
            session.record_miss();
        }
    }

    /// Finish the session and return analytics data
    pub fn finish_session(self) -> Option<crate::analytics::GameSession> {
        self.active_session.map(|s| s.finish())
    }
}

/// End state for results screen
#[derive(Debug, Clone)]
pub struct EndState {
    /// Final score
    pub score: i32,
    /// Max combo
    pub max_combo: u32,
    /// Hit statistics
    pub hits: crate::analytics::HitStats,
    /// Accuracy percentage
    pub accuracy: f32,
    /// Grade achieved
    pub grade: crate::analytics::Grade,
    /// Full combo achieved
    pub full_combo: bool,
    /// Song name
    pub song_name: String,
    /// Whether it was practice mode
    pub practice_mode: bool,
    /// Playback speed
    pub playback_speed: f32,
    /// New best score
    pub new_best: bool,
    /// Previous best score
    pub previous_best: i32,
}

/// Practice menu state
#[derive(Debug, Clone, Resource)]
pub struct PracticeMenuState {
    /// Selected song
    pub selected_song: Option<String>,
    /// Playback speed
    pub playback_speed: f32,
    /// No-fail mode
    pub no_fail: bool,
    /// Autoplay mode
    pub autoplay: bool,
    /// Enable hit sounds
    pub hit_sounds: bool,
    /// Loop start time
    pub loop_start: Option<f64>,
    /// Loop end time
    pub loop_end: Option<f64>,
    /// Selected menu item
    pub selected_index: usize,
}

impl Default for PracticeMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl PracticeMenuState {
    /// Create new practice menu state
    pub fn new() -> Self {
        Self {
            selected_song: None,
            playback_speed: 1.0,
            no_fail: false,
            autoplay: false,
            hit_sounds: true,
            loop_start: None,
            loop_end: None,
            selected_index: 0,
        }
    }

    /// Get playback speed options
    pub fn speed_options() -> Vec<(f32, &'static str)> {
        vec![
            (0.25, "0.25x"),
            (0.5, "0.5x"),
            (0.75, "0.75x"),
            (1.0, "1.0x"),
            (1.25, "1.25x"),
            (1.5, "1.5x"),
            (2.0, "2.0x"),
        ]
    }

    /// Get next speed
    pub fn next_speed(&mut self) {
        let options = Self::speed_options();
        let current_idx = options
            .iter()
            .position(|(s, _)| *s == self.playback_speed)
            .unwrap_or(3);
        let next_idx = (current_idx + 1).min(options.len() - 1);
        self.playback_speed = options[next_idx].0;
    }

    /// Get previous speed
    pub fn previous_speed(&mut self) {
        let options = Self::speed_options();
        let current_idx = options
            .iter()
            .position(|(s, _)| *s == self.playback_speed)
            .unwrap_or(3);
        let prev_idx = current_idx.saturating_sub(1);
        self.playback_speed = options[prev_idx].0;
    }
}

/// Resource to hold the current game state
#[derive(Resource, Default)]
pub struct GameStateResource {
    pub state: GameState,
    pub selected_song: String,
    pub songs: Vec<String>,
}

/// Resource to hold audio sink
#[derive(Resource)]
pub struct GameAudioSink {
    pub sink: rodio::Sink,
}

/// Resource to hold timing information
#[derive(Resource)]
pub struct GameTime {
    pub start_time: Instant,
    pub elapsed: f64,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed: 0.0,
        }
    }
}

/// Resource for loading data - stores only the beats once loaded
#[derive(Resource)]
pub struct LoadingData {
    pub beats: Option<Vec<f64>>,
    pub start_time: Instant,
    pub song_path: String,
}

impl Default for LoadingData {
    fn default() -> Self {
        Self {
            beats: None,
            start_time: Instant::now(),
            song_path: String::new(),
        }
    }
}

/// Resource for ready to play data
#[derive(Resource)]
pub struct ReadyToPlayData {
    pub beats: Vec<f64>,
    pub ready_time: Instant,
}

/// Resource for visualizing data
#[derive(Resource)]
pub struct VisualizingData {
    pub state: VisualizingState,
    pub start_time: Instant,
}

/// Resource for end data
#[derive(Resource)]
pub struct EndData {
    pub state: EndState,
}
