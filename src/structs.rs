// src/structs.rs

use macroquad::prelude::Vec2;
use macroquad::text::Font;
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::time::Instant;
use uuid::Uuid;

use crate::accounts::User;
use crate::analytics::{ActiveSession, Analytics};
use crate::community::Tournament;
use crate::config::GameConfig;
use crate::network::Room;

/// UI Assets container
pub struct Assets {
    pub cyberpunk_font: Font,
}

/// Song selection state
pub struct SongSelectionState {
    pub scroll_pos: f32,
    pub selected_song: Option<String>,
    /// Whether practice mode is enabled
    pub practice_mode: bool,
    /// Selected playback speed for practice mode
    pub playback_speed: f32,
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
pub enum GameState {
    Menu,
    SongSelection,
    /// Practice tools menu
    PracticeMenu,
    /// Multiplayer lobby
    MultiplayerLobby,
    /// Login screen
    Login,
    /// Registration screen
    Register,
    /// Profile view
    Profile,
    /// Leaderboard
    Leaderboard,
    /// Friends list
    Friends,
    /// Community hub
    CommunityHub,
    /// Tournament view
    Tournament,
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

/// Game circle structure
pub struct Circle {
    pub position: Vec2,
    pub spawn_time: f64,
    pub hit_time: f64,
    pub max_radius: f32,
    pub hit: bool,
    pub missed: bool,
}

/// Floating text for feedback
pub struct FloatingText {
    pub text: String,
    pub position: Vec2,
    pub spawn_time: f64,
    pub duration: f64,
    /// Text color
    pub color: (f32, f32, f32),
}

/// Visualizing/gameplay state
pub struct VisualizingState {
    pub beats: Vec<f64>,
    pub start_time: Instant,
    pub circles: Vec<Circle>,
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
        circles: Vec<Circle>,
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
#[derive(Debug, Clone)]
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

/// Login state
#[derive(Debug, Clone)]
pub struct LoginState {
    pub username: String,
    pub password: String,
    pub error_message: Option<String>,
    pub is_registering: bool,
}

impl LoginState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            error_message: None,
            is_registering: false,
        }
    }
}

/// Registration state
#[derive(Debug, Clone)]
pub struct RegisterState {
    pub username: String,
    pub password: String,
    pub email: String,
    pub confirm_password: String,
    pub error_message: Option<String>,
}

impl RegisterState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            email: String::new(),
            confirm_password: String::new(),
            error_message: None,
        }
    }
}

/// Multiplayer lobby state
#[derive(Debug, Clone)]
pub struct MultiplayerLobbyState {
    pub selected_room: Option<Uuid>,
    pub room_password: String,
    pub create_room: bool,
    pub max_players: usize,
    pub room_name: String,
    pub selected_index: usize,
}

impl MultiplayerLobbyState {
    pub fn new() -> Self {
        Self {
            selected_room: None,
            room_password: String::new(),
            create_room: false,
            max_players: 4,
            room_name: String::new(),
            selected_index: 0,
        }
    }
}

/// Profile view state
#[derive(Debug, Clone)]
pub struct ProfileState {
    pub viewing_user_id: Option<Uuid>,
    pub selected_tab: ProfileTab,
}

#[derive(Debug, Clone, Copy)]
pub enum ProfileTab {
    Overview,
    Stats,
    Achievements,
    Scores,
}

impl ProfileState {
    pub fn new() -> Self {
        Self {
            viewing_user_id: None,
            selected_tab: ProfileTab::Overview,
        }
    }
}

/// Leaderboard state
#[derive(Debug, Clone)]
pub struct LeaderboardState {
    pub selected_tab: LeaderboardTab,
    pub country_filter: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum LeaderboardTab {
    Global,
    Country,
    Friends,
}

impl LeaderboardState {
    pub fn new() -> Self {
        Self {
            selected_tab: LeaderboardTab::Global,
            country_filter: None,
        }
    }
}

/// Friends list state
#[derive(Debug, Clone)]
pub struct FriendsState {
    pub selected_index: usize,
    pub searching: bool,
    pub search_query: String,
}

impl FriendsState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            searching: false,
            search_query: String::new(),
        }
    }
}

/// Community hub state
#[derive(Debug, Clone)]
pub struct CommunityHubState {
    pub selected_tab: CommunityTab,
}

#[derive(Debug, Clone, Copy)]
pub enum CommunityTab {
    Tournaments,
    Chat,
    Events,
}

impl CommunityHubState {
    pub fn new() -> Self {
        Self {
            selected_tab: CommunityTab::Tournaments,
        }
    }
}

/// Tournament view state
#[derive(Debug, Clone)]
pub struct TournamentState {
    pub tournament_id: Option<Uuid>,
    pub is_participating: bool,
}

impl TournamentState {
    pub fn new() -> Self {
        Self {
            tournament_id: None,
            is_participating: false,
        }
    }
}

/// Current user session
#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub token: String,
    pub expires_at: std::time::SystemTime,
}

impl UserSession {
    pub fn new(user_id: Uuid, username: String, token: String) -> Self {
        Self {
            user_id,
            username,
            token,
            expires_at: std::time::SystemTime::now()
                + std::time::Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        }
    }

    pub fn is_expired(&self) -> bool {
        std::time::SystemTime::now() > self.expires_at
    }
}
