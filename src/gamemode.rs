// src/gamemode.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// Game mode types that affect how the game is played
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Standard mode - classic gameplay with scoring
    Standard,
    /// Time Attack - score as many points as possible within a time limit
    TimeAttack { time_limit_seconds: u32 },
    /// Precision - emphasizes accuracy over score
    Precision,
    /// Survival - complete the song with limited lives
    Survival { lives: u32 },
    /// Endless - circles keep coming until you miss
    Endless,
    /// Zen - no scoring, just play for fun
    Zen,
}

impl GameMode {
    /// Get all game modes (excluding parameterized variants)
    pub fn all_variants() -> Vec<GameMode> {
        vec![
            GameMode::Standard,
            GameMode::TimeAttack {
                time_limit_seconds: 60,
            },
            GameMode::Precision,
            GameMode::Survival { lives: 3 },
            GameMode::Endless,
            GameMode::Zen,
        ]
    }

    /// Get display name for the game mode
    pub fn display_name(&self) -> &'static str {
        match self {
            GameMode::Standard => "Standard",
            GameMode::TimeAttack { .. } => "Time Attack",
            GameMode::Precision => "Precision",
            GameMode::Survival { .. } => "Survival",
            GameMode::Endless => "Endless",
            GameMode::Zen => "Zen",
        }
    }

    /// Get description for the game mode
    pub fn description(&self) -> &'static str {
        match self {
            GameMode::Standard => "Classic rhythm gameplay",
            GameMode::TimeAttack { .. } => "Score as much as possible in limited time",
            GameMode::Precision => "Focus on accuracy over raw score",
            GameMode::Survival { .. } => "Complete the song with limited lives",
            GameMode::Endless => "Keep going until you miss",
            GameMode::Zen => "No scoring, just play for fun",
        }
    }
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Standard
    }
}

/// Difficulty settings that affect circle behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    /// Easy mode - larger circles, slower approach
    Easy,
    /// Normal mode - standard gameplay
    Normal,
    /// Hard mode - smaller circles, faster approach
    Hard,
    /// Expert mode - very challenging
    Expert,
    /// Insane mode - extreme difficulty
    Insane,
}

impl Difficulty {
    /// Get all difficulty levels
    pub fn all() -> Vec<(Difficulty, &'static str)> {
        vec![
            (Difficulty::Easy, "Easy"),
            (Difficulty::Normal, "Normal"),
            (Difficulty::Hard, "Hard"),
            (Difficulty::Expert, "Expert"),
            (Difficulty::Insane, "Insane"),
        ]
    }

    /// Get display name for the difficulty
    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
            Difficulty::Insane => "Insane",
        }
    }

    /// Get description for the difficulty
    pub fn description(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Larger circles, more forgiving timing",
            Difficulty::Normal => "Standard gameplay",
            Difficulty::Hard => "Smaller circles, faster approach",
            Difficulty::Expert => "Very challenging gameplay",
            Difficulty::Insane => "Extreme difficulty for experts",
        }
    }

    /// Get circle size multiplier for this difficulty
    pub fn circle_size_multiplier(&self) -> f32 {
        match self {
            Difficulty::Easy => 1.3,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 0.85,
            Difficulty::Expert => 0.7,
            Difficulty::Insane => 0.55,
        }
    }

    /// Get shrink time multiplier for this difficulty
    pub fn shrink_time_multiplier(&self) -> f32 {
        match self {
            Difficulty::Easy => 1.3,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 0.85,
            Difficulty::Expert => 0.7,
            Difficulty::Insane => 0.55,
        }
    }

    /// Get score multiplier for this difficulty
    pub fn score_multiplier(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.5,
            Difficulty::Expert => 2.0,
            Difficulty::Insane => 3.0,
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty::Normal
    }
}

/// Game modifiers that change gameplay mechanics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Modifier {
    /// Sudden death - one miss ends the game
    SuddenDeath,
    /// Perfect only - anything below a perfect counts as a miss
    PerfectOnly,
    /// Hidden - approach circles are invisible
    Hidden,
    /// Flash - circles flash visible/invisible
    Flash,
    /// No Fail - game doesn't end on miss
    NoFail,
    /// Auto - game plays itself
    Auto,
    /// Relaxed - no timing judgment, just press any key
    Relaxed,
    /// Randomize - circles appear at random positions
    Randomize,
    /// Double Time - 1.5x playback speed
    DoubleTime,
    /// Half Time - 0.75x playback speed
    HalfTime,
    /// Hard Rock - smaller circles, less forgiving timing
    HardRock,
    /// Easy - larger circles, more forgiving timing
    EasyMod,
}

impl Modifier {
    /// Get all available modifiers
    pub fn all() -> Vec<(Modifier, &'static str)> {
        vec![
            (Modifier::SuddenDeath, "Sudden Death"),
            (Modifier::PerfectOnly, "Perfect Only"),
            (Modifier::Hidden, "Hidden"),
            (Modifier::Flash, "Flash"),
            (Modifier::NoFail, "No Fail"),
            (Modifier::Auto, "Auto"),
            (Modifier::Relaxed, "Relaxed"),
            (Modifier::Randomize, "Randomize"),
            (Modifier::DoubleTime, "Double Time"),
            (Modifier::HalfTime, "Half Time"),
            (Modifier::HardRock, "Hard Rock"),
            (Modifier::EasyMod, "Easy"),
        ]
    }

    /// Get display name for the modifier
    pub fn display_name(&self) -> &'static str {
        match self {
            Modifier::SuddenDeath => "Sudden Death",
            Modifier::PerfectOnly => "Perfect Only",
            Modifier::Hidden => "Hidden",
            Modifier::Flash => "Flash",
            Modifier::NoFail => "No Fail",
            Modifier::Auto => "Auto",
            Modifier::Relaxed => "Relaxed",
            Modifier::Randomize => "Randomize",
            Modifier::DoubleTime => "Double Time",
            Modifier::HalfTime => "Half Time",
            Modifier::HardRock => "Hard Rock",
            Modifier::EasyMod => "Easy",
        }
    }

    /// Get description for the modifier
    pub fn description(&self) -> &'static str {
        match self {
            Modifier::SuddenDeath => "One miss ends the game",
            Modifier::PerfectOnly => "Only perfect hits count",
            Modifier::Hidden => "Approach circles are invisible",
            Modifier::Flash => "Circles flash visible/invisible",
            Modifier::NoFail => "Game doesn't end on miss",
            Modifier::Auto => "Game plays itself",
            Modifier::Relaxed => "No timing judgment",
            Modifier::Randomize => "Circles appear at random positions",
            Modifier::DoubleTime => "1.5x playback speed",
            Modifier::HalfTime => "0.75x playback speed",
            Modifier::HardRock => "Smaller circles, less forgiving",
            Modifier::EasyMod => "Larger circles, more forgiving",
        }
    }

    /// Get score multiplier for this modifier
    pub fn score_multiplier(&self) -> f32 {
        match self {
            Modifier::SuddenDeath => 2.0,
            Modifier::PerfectOnly => 2.0,
            Modifier::Hidden => 1.5,
            Modifier::Flash => 1.3,
            Modifier::NoFail => 0.5,
            Modifier::Auto => 0.0,
            Modifier::Relaxed => 0.0,
            Modifier::Randomize => 1.2,
            Modifier::DoubleTime => 1.5,
            Modifier::HalfTime => 0.5,
            Modifier::HardRock => 1.5,
            Modifier::EasyMod => 0.5,
        }
    }

    /// Check if modifier conflicts with another modifier
    pub fn conflicts_with(&self, other: &Modifier) -> bool {
        match self {
            Modifier::DoubleTime => matches!(other, Modifier::HalfTime),
            Modifier::HalfTime => matches!(other, Modifier::DoubleTime),
            Modifier::HardRock => matches!(other, Modifier::EasyMod),
            Modifier::EasyMod => matches!(other, Modifier::HardRock),
            Modifier::SuddenDeath => matches!(other, Modifier::NoFail),
            Modifier::NoFail => matches!(other, Modifier::SuddenDeath),
            Modifier::Auto => !matches!(other, Modifier::Auto),
            Modifier::Relaxed => matches!(other, Modifier::PerfectOnly),
            Modifier::PerfectOnly => matches!(other, Modifier::Relaxed),
            _ => false,
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Game settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Game mode
    pub mode: GameMode,
    /// Difficulty level
    pub difficulty: Difficulty,
    /// Active modifiers
    pub modifiers: Vec<Modifier>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: GameMode::Standard,
            difficulty: Difficulty::Normal,
            modifiers: Vec::new(),
        }
    }
}

impl GameSettings {
    /// Create new game settings
    pub fn new(mode: GameMode, difficulty: Difficulty) -> Self {
        Self {
            mode,
            difficulty,
            modifiers: Vec::new(),
        }
    }

    /// Add a modifier
    pub fn add_modifier(&mut self, modifier: Modifier) -> Result<(), String> {
        // Check for conflicts
        for existing in &self.modifiers {
            if modifier.conflicts_with(existing) {
                return Err(format!(
                    "{} conflicts with {}",
                    modifier.display_name(),
                    existing.display_name()
                ));
            }
        }
        // Check if already present
        if self.modifiers.contains(&modifier) {
            return Err(format!("{} is already active", modifier.display_name()));
        }
        self.modifiers.push(modifier);
        Ok(())
    }

    /// Remove a modifier
    pub fn remove_modifier(&mut self, modifier: Modifier) {
        self.modifiers.retain(|m| *m != modifier);
    }

    /// Calculate total score multiplier
    pub fn score_multiplier(&self) -> f32 {
        let difficulty_mult = self.difficulty.score_multiplier();
        let modifier_mult: f32 = self
            .modifiers
            .iter()
            .map(|m| m.score_multiplier())
            .product();
        difficulty_mult * modifier_mult
    }

    /// Check if a modifier is active
    pub fn has_modifier(&self, modifier: Modifier) -> bool {
        self.modifiers.contains(&modifier)
    }

    /// Get the effective playback speed
    pub fn playback_speed(&self) -> f32 {
        if self.has_modifier(Modifier::DoubleTime) {
            1.5
        } else if self.has_modifier(Modifier::HalfTime) {
            0.75
        } else {
            1.0
        }
    }

    /// Check if the game should end on miss
    pub fn end_on_miss(&self) -> bool {
        self.has_modifier(Modifier::SuddenDeath)
            || (matches!(self.mode, GameMode::Survival { .. })
                && !self.has_modifier(Modifier::NoFail))
    }

    /// Check if approach circles should be visible
    pub fn show_approach_circles(&self) -> bool {
        !self.has_modifier(Modifier::Hidden)
    }

    /// Check if the game is in auto-play mode
    pub fn is_auto(&self) -> bool {
        self.has_modifier(Modifier::Auto)
    }

    /// Check if timing judgment is disabled
    pub fn is_relaxed(&self) -> bool {
        self.has_modifier(Modifier::Relaxed)
    }

    /// Check if only perfect hits count
    pub fn perfect_only(&self) -> bool {
        self.has_modifier(Modifier::PerfectOnly)
    }

    /// Check if circles should be randomized
    pub fn randomize_positions(&self) -> bool {
        self.has_modifier(Modifier::Randomize)
    }
}
