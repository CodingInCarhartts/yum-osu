// src/analytics.rs

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Analytics data for tracking player performance
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Analytics {
    /// Unique player identifier
    pub player_id: String,
    /// Total play time in seconds
    pub total_play_time_seconds: u64,
    /// Total number of games played
    pub total_games_played: u32,
    /// Total hits (all types combined)
    pub total_hits: HitStats,
    /// Statistics per song
    pub song_stats: HashMap<String, SongStats>,
    /// Recent sessions (last 50)
    pub recent_sessions: Vec<GameSession>,
    /// Overall accuracy history
    pub accuracy_history: Vec<f32>,
    /// Best scores per song
    pub best_scores: HashMap<String, i32>,
    /// Achievements unlocked
    pub achievements: Vec<Achievement>,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Hit statistics for tracking different hit types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HitStats {
    /// Perfect hits (300 points)
    pub perfect: u32,
    /// Good hits (100 points)
    pub good: u32,
    /// Okay hits (50 points)
    pub okay: u32,
    /// Misses (0 points)
    pub misses: u32,
}

impl HitStats {
    /// Create new empty hit stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total hits
    pub fn total(&self) -> u32 {
        self.perfect + self.good + self.okay + self.misses
    }

    /// Get accuracy percentage (0.0 - 100.0)
    pub fn accuracy(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        let weighted_score =
            (self.perfect as f32 * 300.0) + (self.good as f32 * 100.0) + (self.okay as f32 * 50.0);
        let max_score = total as f32 * 300.0;
        (weighted_score / max_score) * 100.0
    }

    /// Get hit rate percentage (non-misses)
    pub fn hit_rate(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        ((total - self.misses) as f32 / total as f32) * 100.0
    }

    /// Add hits from a session
    pub fn add_session(&mut self, session: &HitStats) {
        self.perfect += session.perfect;
        self.good += session.good;
        self.okay += session.okay;
        self.misses += session.misses;
    }

    /// Get grade based on accuracy
    pub fn grade(&self) -> Grade {
        let accuracy = self.accuracy();
        if accuracy >= 95.0 {
            Grade::SS
        } else if accuracy >= 90.0 {
            Grade::S
        } else if accuracy >= 80.0 {
            Grade::A
        } else if accuracy >= 70.0 {
            Grade::B
        } else if accuracy >= 60.0 {
            Grade::C
        } else {
            Grade::D
        }
    }
}

/// Performance grade
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Grade {
    SS,
    S,
    A,
    B,
    C,
    D,
    F,
}

impl Grade {
    /// Get grade as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::SS => "SS",
            Grade::S => "S",
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }

    /// Get grade color
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            Grade::SS => (1.0, 0.84, 0.0), // Gold
            Grade::S => (1.0, 0.5, 0.0),   // Orange
            Grade::A => (0.0, 1.0, 0.0),   // Green
            Grade::B => (0.0, 0.5, 1.0),   // Blue
            Grade::C => (0.5, 0.0, 1.0),   // Purple
            Grade::D => (1.0, 0.0, 0.5),   // Pink
            Grade::F => (1.0, 0.0, 0.0),   // Red
        }
    }
}

/// Statistics for a specific song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongStats {
    /// Song name
    pub song_name: String,
    /// Number of times played
    pub play_count: u32,
    /// Best score achieved
    pub best_score: i32,
    /// Best accuracy achieved
    pub best_accuracy: f32,
    /// Total hits for this song
    pub total_hits: HitStats,
    /// Average score
    pub average_score: f32,
    /// Total play time in seconds
    pub total_play_time_seconds: u64,
}

impl SongStats {
    /// Create new song stats
    pub fn new(song_name: String) -> Self {
        Self {
            song_name,
            play_count: 0,
            best_score: 0,
            best_accuracy: 0.0,
            total_hits: HitStats::new(),
            average_score: 0.0,
            total_play_time_seconds: 0,
        }
    }

    /// Update with a new session
    pub fn update(&mut self, session: &GameSession) {
        self.play_count += 1;
        self.total_play_time_seconds += session.duration_seconds;
        self.total_hits.add_session(&session.hits);

        if session.score > self.best_score {
            self.best_score = session.score;
        }

        let session_accuracy = session.hits.accuracy();
        if session_accuracy > self.best_accuracy {
            self.best_accuracy = session_accuracy;
        }

        // Update average score
        let total_score = self.average_score * (self.play_count - 1) as f32;
        self.average_score = (total_score + session.score as f32) / self.play_count as f32;
    }
}

/// Individual game session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Session ID (timestamp)
    pub session_id: u64,
    /// Song name
    pub song_name: String,
    /// Score achieved
    pub score: i32,
    /// Hit statistics
    pub hits: HitStats,
    /// Duration in seconds
    pub duration_seconds: u64,
    /// Accuracy percentage
    pub accuracy: f32,
    /// Grade achieved
    pub grade: Grade,
    /// Whether it was a full combo (no misses)
    pub full_combo: bool,
    /// Whether practice mode was enabled
    pub practice_mode: bool,
    /// Playback speed if in practice mode
    pub playback_speed: Option<f32>,
}

impl GameSession {
    /// Create a new game session
    pub fn new(song_name: String) -> Self {
        Self {
            session_id: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            song_name,
            score: 0,
            hits: HitStats::new(),
            duration_seconds: 0,
            accuracy: 0.0,
            grade: Grade::F,
            full_combo: false,
            practice_mode: false,
            playback_speed: None,
        }
    }
}

/// Achievement structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    /// Achievement ID
    pub id: String,
    /// Achievement name
    pub name: String,
    /// Achievement description
    pub description: String,
    /// When it was unlocked
    pub unlocked_at: SystemTime,
    /// Achievement icon/category
    pub category: AchievementCategory,
}

/// Achievement categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCategory {
    Accuracy,
    Score,
    Streak,
    Songs,
    Special,
}

impl AchievementCategory {
    /// Get category name
    pub fn name(&self) -> &'static str {
        match self {
            AchievementCategory::Accuracy => "Accuracy",
            AchievementCategory::Score => "Score",
            AchievementCategory::Streak => "Streak",
            AchievementCategory::Songs => "Songs",
            AchievementCategory::Special => "Special",
        }
    }
}

/// Active session for tracking current game
#[derive(Debug, Clone)]
pub struct ActiveSession {
    /// Session start time
    pub start_time: std::time::Instant,
    /// Current hit stats
    pub hits: HitStats,
    /// Current score
    pub score: i32,
    /// Song name
    pub song_name: String,
    /// Whether practice mode is enabled
    pub practice_mode: bool,
    /// Playback speed
    pub playback_speed: f32,
    /// Hit timings for precision analysis (in milliseconds)
    pub hit_timings: Vec<f32>,
}

impl ActiveSession {
    /// Create a new active session
    pub fn new(song_name: String, practice_mode: bool, playback_speed: f32) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            hits: HitStats::new(),
            score: 0,
            song_name,
            practice_mode,
            playback_speed,
            hit_timings: Vec::new(),
        }
    }

    /// Record a hit
    pub fn record_hit(&mut self, points: i32, timing_ms: f32) {
        self.score += points;
        self.hit_timings.push(timing_ms);

        match points {
            300 => self.hits.perfect += 1,
            100 => self.hits.good += 1,
            50 => self.hits.okay += 1,
            _ => self.hits.misses += 1,
        }
    }

    /// Record a miss
    pub fn record_miss(&mut self) {
        self.hits.misses += 1;
    }

    /// Finish the session and create a GameSession
    pub fn finish(self) -> GameSession {
        let duration = self.start_time.elapsed().as_secs();
        let accuracy = self.hits.accuracy();
        let full_combo = self.hits.misses == 0;

        GameSession {
            session_id: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            song_name: self.song_name,
            score: self.score,
            hits: self.hits.clone(),
            duration_seconds: duration,
            accuracy,
            grade: self.hits.grade(),
            full_combo,
            practice_mode: self.practice_mode,
            playback_speed: if self.practice_mode {
                Some(self.playback_speed)
            } else {
                None
            },
        }
    }

    /// Get current accuracy
    pub fn current_accuracy(&self) -> f32 {
        self.hits.accuracy()
    }

    /// Get timing statistics (mean, std deviation)
    pub fn timing_stats(&self) -> Option<(f32, f32)> {
        if self.hit_timings.is_empty() {
            return None;
        }

        let mean = self.hit_timings.iter().sum::<f32>() / self.hit_timings.len() as f32;
        let variance = self
            .hit_timings
            .iter()
            .map(|&t| (t - mean).powi(2))
            .sum::<f32>()
            / self.hit_timings.len() as f32;
        let std_dev = variance.sqrt();

        Some((mean, std_dev))
    }
}

impl Default for Analytics {
    fn default() -> Self {
        Self {
            player_id: generate_player_id(),
            total_play_time_seconds: 0,
            total_games_played: 0,
            total_hits: HitStats::new(),
            song_stats: HashMap::new(),
            recent_sessions: Vec::new(),
            accuracy_history: Vec::new(),
            best_scores: HashMap::new(),
            achievements: Vec::new(),
            last_updated: SystemTime::now(),
        }
    }
}

impl Analytics {
    /// Load analytics from file or create default
    pub fn load() -> Self {
        let analytics_path = "analytics.json";
        if Path::new(analytics_path).exists() {
            match fs::read_to_string(analytics_path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(analytics) => analytics,
                    Err(e) => {
                        eprintln!("Failed to parse analytics: {}, using default", e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read analytics: {}, using default", e);
                    Self::default()
                }
            }
        } else {
            let analytics = Self::default();
            analytics.save();
            analytics
        }
    }

    /// Save analytics to file
    pub fn save(&self) {
        let analytics_path = "analytics.json";
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(analytics_path, json) {
                    eprintln!("Failed to save analytics: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize analytics: {}", e);
            }
        }
    }

    /// Add a completed game session
    pub fn add_session(&mut self, session: GameSession) {
        self.total_games_played += 1;
        self.total_play_time_seconds += session.duration_seconds;
        self.total_hits.add_session(&session.hits);
        self.accuracy_history.push(session.accuracy);

        // Keep only last 100 accuracy values
        if self.accuracy_history.len() > 100 {
            self.accuracy_history.remove(0);
        }

        // Update song stats
        let song_stats = self
            .song_stats
            .entry(session.song_name.clone())
            .or_insert_with(|| SongStats::new(session.song_name.clone()));
        song_stats.update(&session);

        // Update best score
        if session.score > *self.best_scores.get(&session.song_name).unwrap_or(&0) {
            self.best_scores
                .insert(session.song_name.clone(), session.score);
        }

        // Add to recent sessions
        self.recent_sessions.push(session);

        // Keep only last 50 sessions
        if self.recent_sessions.len() > 50 {
            self.recent_sessions.remove(0);
        }

        // Check for achievements
        self.check_achievements();

        self.last_updated = SystemTime::now();
        self.save();
    }

    /// Check and unlock achievements
    fn check_achievements(&mut self) {
        let achievements_to_check = vec![
            (
                "first_game",
                "First Steps",
                "Play your first game",
                AchievementCategory::Special,
                1u32,
            ),
            (
                "ten_games",
                "Getting Started",
                "Play 10 games",
                AchievementCategory::Special,
                10u32,
            ),
            (
                "hundred_games",
                "Rhythm Master",
                "Play 100 games",
                AchievementCategory::Special,
                100u32,
            ),
            (
                "perfect_accuracy",
                "Perfect",
                "Achieve 100% accuracy",
                AchievementCategory::Accuracy,
                0u32,
            ),
            (
                "ss_grade",
                "SS Rank",
                "Get an SS grade",
                AchievementCategory::Score,
                0u32,
            ),
            (
                "full_combo",
                "Full Combo",
                "Complete a song without misses",
                AchievementCategory::Streak,
                0u32,
            ),
        ];

        for (id, name, desc, category, threshold) in achievements_to_check {
            if !self.has_achievement(id) {
                let should_unlock = match id {
                    "first_game" | "ten_games" | "hundred_games" => {
                        self.total_games_played >= threshold
                    }
                    "perfect_accuracy" => self.accuracy_history.iter().any(|&a| a >= 100.0),
                    "ss_grade" => self.recent_sessions.iter().any(|s| s.grade == Grade::SS),
                    "full_combo" => self.recent_sessions.iter().any(|s| s.full_combo),
                    _ => false,
                };

                if should_unlock {
                    self.unlock_achievement(id, name, desc, category);
                }
            }
        }
    }

    /// Check if player has an achievement
    fn has_achievement(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == id)
    }

    /// Unlock an achievement
    fn unlock_achievement(
        &mut self,
        id: &str,
        name: &str,
        description: &str,
        category: AchievementCategory,
    ) {
        self.achievements.push(Achievement {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            unlocked_at: SystemTime::now(),
            category,
        });
    }

    /// Get overall statistics
    pub fn get_overall_stats(&self) -> OverallStats {
        OverallStats {
            total_games: self.total_games_played,
            total_play_time: self.total_play_time_seconds,
            overall_accuracy: self.total_hits.accuracy(),
            average_score: if self.total_games_played > 0 {
                self.total_hits.total() as f32 * self.total_hits.accuracy() / 100.0
            } else {
                0.0
            },
            best_overall_grade: self.get_best_grade(),
            total_full_combos: self.recent_sessions.iter().filter(|s| s.full_combo).count() as u32,
        }
    }

    /// Get best grade achieved
    fn get_best_grade(&self) -> Option<Grade> {
        self.recent_sessions
            .iter()
            .map(|s| s.grade.clone())
            .min_by_key(|g| match g {
                Grade::SS => 0,
                Grade::S => 1,
                Grade::A => 2,
                Grade::B => 3,
                Grade::C => 4,
                Grade::D => 5,
                Grade::F => 6,
            })
    }

    /// Get accuracy trend (last 10 sessions)
    pub fn get_accuracy_trend(&self) -> Vec<f32> {
        self.accuracy_history
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect()
    }

    /// Get most played songs
    pub fn get_most_played_songs(&self, limit: usize) -> Vec<(&String, &SongStats)> {
        let mut songs: Vec<_> = self.song_stats.iter().collect();
        songs.sort_by(|a, b| b.1.play_count.cmp(&a.1.play_count));
        songs.into_iter().take(limit).collect()
    }
}

/// Overall statistics summary
#[derive(Debug, Clone)]
pub struct OverallStats {
    pub total_games: u32,
    pub total_play_time: u64,
    pub overall_accuracy: f32,
    pub average_score: f32,
    pub best_overall_grade: Option<Grade>,
    pub total_full_combos: u32,
}

impl OverallStats {
    /// Format play time as human readable string
    pub fn format_play_time(&self) -> String {
        let hours = self.total_play_time / 3600;
        let minutes = (self.total_play_time % 3600) / 60;
        let seconds = self.total_play_time % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Generate a unique player ID
fn generate_player_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen();
    format!("player_{:016x}", id)
}

/// Analytics display state
#[derive(Debug, Clone, Resource)]
pub struct AnalyticsState {
    /// Current view tab
    pub current_view: AnalyticsView,
    /// Selected song for detailed view
    pub selected_song: Option<String>,
    /// Scroll position
    pub scroll_y: f32,
    /// Selected session index
    pub selected_session: Option<usize>,
}

impl AnalyticsState {
    /// Create new analytics state
    pub fn new() -> Self {
        Self {
            current_view: AnalyticsView::Overview,
            selected_song: None,
            scroll_y: 0.0,
            selected_session: None,
        }
    }
}

/// Analytics view tabs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalyticsView {
    Overview,
    Songs,
    Sessions,
    Achievements,
    Trends,
}

impl AnalyticsView {
    /// Get all views
    pub fn all() -> Vec<(AnalyticsView, &'static str)> {
        vec![
            (AnalyticsView::Overview, "Overview"),
            (AnalyticsView::Songs, "Songs"),
            (AnalyticsView::Sessions, "Sessions"),
            (AnalyticsView::Achievements, "Achievements"),
            (AnalyticsView::Trends, "Trends"),
        ]
    }

    /// Get next view
    pub fn next(&self) -> AnalyticsView {
        match self {
            AnalyticsView::Overview => AnalyticsView::Songs,
            AnalyticsView::Songs => AnalyticsView::Sessions,
            AnalyticsView::Sessions => AnalyticsView::Achievements,
            AnalyticsView::Achievements => AnalyticsView::Trends,
            AnalyticsView::Trends => AnalyticsView::Overview,
        }
    }

    /// Get previous view
    pub fn previous(&self) -> AnalyticsView {
        match self {
            AnalyticsView::Overview => AnalyticsView::Trends,
            AnalyticsView::Songs => AnalyticsView::Overview,
            AnalyticsView::Sessions => AnalyticsView::Songs,
            AnalyticsView::Achievements => AnalyticsView::Sessions,
            AnalyticsView::Trends => AnalyticsView::Achievements,
        }
    }
}
