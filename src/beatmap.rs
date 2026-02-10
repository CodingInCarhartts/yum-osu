// src/beatmap.rs

use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Beatmap version for compatibility
pub const BEATMAP_VERSION: u32 = 1;

/// Hit object types for different gameplay elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitObjectType {
    /// Standard circle hit
    Circle,
    /// Slider (hold and follow)
    Slider,
    /// Spinner (rapid hits in area)
    Spinner,
}

impl Default for HitObjectType {
    fn default() -> Self {
        HitObjectType::Circle
    }
}

/// A single hit object in the beatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitObject {
    /// Time in seconds when this object should be hit
    pub time: f64,
    /// Position on screen (normalized 0.0-1.0, converted to actual pixels during gameplay)
    pub position: Vec2,
    /// Type of hit object
    #[serde(default)]
    pub object_type: HitObjectType,
    /// For sliders: duration in seconds
    #[serde(default)]
    pub duration: Option<f64>,
    /// For sliders: end position
    #[serde(default)]
    pub end_position: Option<Vec2>,
    /// For sliders: control points for curved paths
    #[serde(default)]
    pub control_points: Option<Vec<Vec2>>,
    /// For spinners: duration in seconds
    #[serde(default)]
    pub spinner_duration: Option<f64>,
    /// New combo indicator (starts a new combo color)
    #[serde(default)]
    pub new_combo: bool,
    /// Hit sound sample index (0 = normal, 1 = whistle, 2 = finish, 3 = clap)
    #[serde(default)]
    pub hit_sound: u8,
}

impl HitObject {
    /// Create a new circle hit object
    pub fn new_circle(time: f64, x: f32, y: f32) -> Self {
        Self {
            time,
            position: Vec2::new(x, y),
            object_type: HitObjectType::Circle,
            duration: None,
            end_position: None,
            control_points: None,
            spinner_duration: None,
            new_combo: false,
            hit_sound: 0,
        }
    }

    /// Create a new slider hit object
    pub fn new_slider(time: f64, start: Vec2, end: Vec2, duration: f64) -> Self {
        Self {
            time,
            position: start,
            object_type: HitObjectType::Slider,
            duration: Some(duration),
            end_position: Some(end),
            control_points: None,
            spinner_duration: None,
            new_combo: false,
            hit_sound: 0,
        }
    }

    /// Create a new spinner hit object
    pub fn new_spinner(time: f64, duration: f64) -> Self {
        Self {
            time,
            position: Vec2::new(0.5, 0.5), // Center of screen
            object_type: HitObjectType::Spinner,
            duration: None,
            end_position: None,
            control_points: None,
            spinner_duration: Some(duration),
            new_combo: false,
            hit_sound: 0,
        }
    }

    /// Get the end time of this object
    pub fn end_time(&self) -> f64 {
        match self.object_type {
            HitObjectType::Circle => self.time,
            HitObjectType::Slider => self.time + self.duration.unwrap_or(0.0),
            HitObjectType::Spinner => self.time + self.spinner_duration.unwrap_or(0.0),
        }
    }
}

/// Timing point for BPM and timing changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingPoint {
    /// Time in seconds when this timing point takes effect
    pub time: f64,
    /// Beats per minute
    pub bpm: f64,
    /// Time signature numerator (e.g., 4 for 4/4)
    pub meter: u8,
    /// Whether this is an inherited timing point (for volume/sample changes)
    #[serde(default)]
    pub inherited: bool,
    /// Volume percentage (0-100)
    #[serde(default = "default_volume")]
    pub volume: u8,
}

fn default_volume() -> u8 {
    100
}

impl TimingPoint {
    /// Create a new timing point
    pub fn new(time: f64, bpm: f64, meter: u8) -> Self {
        Self {
            time,
            bpm,
            meter,
            inherited: false,
            volume: 100,
        }
    }

    /// Get the beat duration in seconds
    pub fn beat_duration(&self) -> f64 {
        60.0 / self.bpm
    }
}

/// Break period where no hit objects appear
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakPeriod {
    /// Start time in seconds
    pub start_time: f64,
    /// End time in seconds
    pub end_time: f64,
}

impl BreakPeriod {
    /// Create a new break period
    pub fn new(start_time: f64, end_time: f64) -> Self {
        Self {
            start_time,
            end_time,
        }
    }

    /// Get the duration of the break
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }
}

/// Beatmap difficulty settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultySettings {
    /// Circle size (CS) - affects circle radius (0-10)
    #[serde(default = "default_circle_size")]
    pub circle_size: f32,
    /// Approach rate (AR) - affects how early circles appear (0-10)
    #[serde(default = "default_approach_rate")]
    pub approach_rate: f32,
    /// Overall difficulty (OD) - affects timing windows (0-10)
    #[serde(default = "default_overall_difficulty")]
    pub overall_difficulty: f32,
    /// HP drain rate (HP) - affects health drain (0-10)
    #[serde(default = "default_hp_drain")]
    pub hp_drain: f32,
    /// Slider multiplier (affects slider speed)
    #[serde(default = "default_slider_multiplier")]
    pub slider_multiplier: f32,
    /// Slider tick rate
    #[serde(default = "default_slider_tick_rate")]
    pub slider_tick_rate: f32,
}

fn default_circle_size() -> f32 {
    5.0
}
fn default_approach_rate() -> f32 {
    5.0
}
fn default_overall_difficulty() -> f32 {
    5.0
}
fn default_hp_drain() -> f32 {
    5.0
}
fn default_slider_multiplier() -> f32 {
    1.4
}
fn default_slider_tick_rate() -> f32 {
    1.0
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self {
            circle_size: default_circle_size(),
            approach_rate: default_approach_rate(),
            overall_difficulty: default_overall_difficulty(),
            hp_drain: default_hp_drain(),
            slider_multiplier: default_slider_multiplier(),
            slider_tick_rate: default_slider_tick_rate(),
        }
    }
}

impl DifficultySettings {
    /// Calculate the approach time in seconds based on AR
    pub fn approach_time(&self) -> f64 {
        // AR 5 = 1.2s, AR 9 = 0.6s, AR 1 = 1.8s
        1.8 - (self.approach_rate as f64 * 0.15)
    }

    /// Calculate timing windows based on OD
    pub fn timing_windows(&self) -> TimingWindows {
        let base = 0.5 - (self.overall_difficulty as f64 * 0.04);
        TimingWindows {
            perfect: base * 0.2, // 300
            good: base * 0.6,    // 100
            okay: base,          // 50
        }
    }

    /// Calculate circle radius based on CS
    pub fn circle_radius(&self) -> f32 {
        // CS 5 = 50px radius, CS 0 = 70px, CS 10 = 30px
        70.0 - (self.circle_size * 4.0)
    }
}

/// Timing windows for hit accuracy
#[derive(Debug, Clone, Copy)]
pub struct TimingWindows {
    /// Perfect hit window in seconds
    pub perfect: f64,
    /// Good hit window in seconds
    pub good: f64,
    /// Okay hit window in seconds
    pub okay: f64,
}

/// Beatmap metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapMetadata {
    /// Beatmap format version
    pub version: u32,
    /// Song title
    pub title: String,
    /// Song artist
    pub artist: String,
    /// Beatmap creator
    pub creator: String,
    /// Beatmap version/difficulty name
    pub version_name: String,
    /// Audio file name
    pub audio_file: String,
    /// Background image file name
    #[serde(default)]
    pub background_file: Option<String>,
    /// Preview start time in seconds
    #[serde(default)]
    pub preview_time: f64,
    /// Tags for searching
    #[serde(default)]
    pub tags: Vec<String>,
    /// Source (anime, game, etc.)
    #[serde(default)]
    pub source: String,
}

impl Default for BeatmapMetadata {
    fn default() -> Self {
        Self {
            version: BEATMAP_VERSION,
            title: "Unknown".to_string(),
            artist: "Unknown".to_string(),
            creator: "Unknown".to_string(),
            version_name: "Normal".to_string(),
            audio_file: "audio.mp3".to_string(),
            background_file: None,
            preview_time: 0.0,
            tags: Vec::new(),
            source: String::new(),
        }
    }
}

/// Color for combo colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ComboColor {
    /// Create a new combo color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to macroquad Color
    pub fn to_color(&self) -> macroquad::prelude::Color {
        macroquad::prelude::Color::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            1.0,
        )
    }
}

/// Complete beatmap data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beatmap {
    /// Beatmap metadata
    pub metadata: BeatmapMetadata,
    /// Difficulty settings
    #[serde(default)]
    pub difficulty: DifficultySettings,
    /// Timing points
    pub timing_points: Vec<TimingPoint>,
    /// Hit objects (sorted by time)
    pub hit_objects: Vec<HitObject>,
    /// Break periods
    #[serde(default)]
    pub breaks: Vec<BreakPeriod>,
    /// Combo colors
    #[serde(default = "default_combo_colors")]
    pub combo_colors: Vec<ComboColor>,
    /// Editor bookmarks (time markers)
    #[serde(default)]
    pub bookmarks: Vec<f64>,
}

fn default_combo_colors() -> Vec<ComboColor> {
    vec![
        ComboColor::new(0, 255, 255), // Cyan
        ComboColor::new(255, 0, 255), // Magenta
        ComboColor::new(255, 255, 0), // Yellow
        ComboColor::new(0, 255, 0),   // Green
        ComboColor::new(0, 128, 255), // Blue
    ]
}

impl Beatmap {
    /// Create a new empty beatmap
    pub fn new(title: String, artist: String, audio_file: String) -> Self {
        Self {
            metadata: BeatmapMetadata {
                title,
                artist,
                audio_file,
                ..Default::default()
            },
            difficulty: DifficultySettings::default(),
            timing_points: vec![TimingPoint::new(0.0, 120.0, 4)],
            hit_objects: Vec::new(),
            breaks: Vec::new(),
            combo_colors: default_combo_colors(),
            bookmarks: Vec::new(),
        }
    }

    /// Add a hit object and maintain sorted order
    pub fn add_hit_object(&mut self, object: HitObject) {
        // Insert while maintaining sorted order by time
        let idx = self
            .hit_objects
            .binary_search_by(|o| o.time.partial_cmp(&object.time).unwrap())
            .unwrap_or_else(|x| x);
        self.hit_objects.insert(idx, object);
    }

    /// Remove a hit object at the given index
    pub fn remove_hit_object(&mut self, index: usize) -> Option<HitObject> {
        if index < self.hit_objects.len() {
            Some(self.hit_objects.remove(index))
        } else {
            None
        }
    }

    /// Get hit objects within a time range
    pub fn get_objects_in_range(&self, start_time: f64, end_time: f64) -> Vec<&HitObject> {
        self.hit_objects
            .iter()
            .filter(|o| o.time >= start_time && o.time <= end_time)
            .collect()
    }

    /// Get the active timing point at a given time
    pub fn get_timing_point_at(&self, time: f64) -> &TimingPoint {
        self.timing_points
            .iter()
            .rfind(|tp| tp.time <= time)
            .unwrap_or(&self.timing_points[0])
    }

    /// Calculate the song duration based on last hit object
    pub fn duration(&self) -> f64 {
        self.hit_objects.last().map(|o| o.end_time()).unwrap_or(0.0)
    }

    /// Get total object count
    pub fn object_count(&self) -> usize {
        self.hit_objects.len()
    }

    /// Get circle count only
    pub fn circle_count(&self) -> usize {
        self.hit_objects
            .iter()
            .filter(|o| o.object_type == HitObjectType::Circle)
            .count()
    }

    /// Get slider count
    pub fn slider_count(&self) -> usize {
        self.hit_objects
            .iter()
            .filter(|o| o.object_type == HitObjectType::Slider)
            .count()
    }

    /// Sort all hit objects by time
    pub fn sort_hit_objects(&mut self) {
        self.hit_objects.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Validate the beatmap for errors
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.metadata.audio_file.is_empty() {
            errors.push("No audio file specified".to_string());
        }

        if self.hit_objects.is_empty() {
            errors.push("No hit objects in beatmap".to_string());
        }

        if self.timing_points.is_empty() {
            errors.push("No timing points defined".to_string());
        }

        // Check for overlapping objects (too close together)
        for i in 1..self.hit_objects.len() {
            let prev = &self.hit_objects[i - 1];
            let curr = &self.hit_objects[i];
            if curr.time - prev.time < 0.05 {
                errors.push(format!(
                    "Objects at {:.2}s and {:.2}s are too close",
                    prev.time, curr.time
                ));
            }
        }

        // Check for objects outside playfield
        for obj in &self.hit_objects {
            if obj.position.x < 0.0 || obj.position.x > 1.0 {
                errors.push(format!("Object at {:.2}s has invalid X position", obj.time));
            }
            if obj.position.y < 0.0 || obj.position.y > 1.0 {
                errors.push(format!("Object at {:.2}s has invalid Y position", obj.time));
            }
        }

        errors
    }
}

/// Beatmap statistics for display
#[derive(Debug, Clone)]
pub struct BeatmapStats {
    pub total_objects: usize,
    pub circles: usize,
    pub sliders: usize,
    pub spinners: usize,
    pub duration_seconds: f64,
    pub average_bpm: f64,
    pub max_combo: u32,
}

impl BeatmapStats {
    /// Calculate statistics from a beatmap
    pub fn from_beatmap(beatmap: &Beatmap) -> Self {
        let mut max_combo = 0u32;
        let mut current_combo = 0u32;

        for obj in &beatmap.hit_objects {
            if obj.new_combo {
                max_combo = max_combo.max(current_combo);
                current_combo = 1;
            } else {
                current_combo += 1;
            }
        }
        max_combo = max_combo.max(current_combo);

        // Calculate average BPM from timing points
        let avg_bpm = if beatmap.timing_points.is_empty() {
            0.0
        } else {
            beatmap.timing_points.iter().map(|tp| tp.bpm).sum::<f64>()
                / beatmap.timing_points.len() as f64
        };

        Self {
            total_objects: beatmap.object_count(),
            circles: beatmap.circle_count(),
            sliders: beatmap.slider_count(),
            spinners: beatmap
                .hit_objects
                .iter()
                .filter(|o| o.object_type == HitObjectType::Spinner)
                .count(),
            duration_seconds: beatmap.duration(),
            average_bpm: avg_bpm,
            max_combo,
        }
    }
}

/// Utility functions for beatmap creation
pub mod utils {
    use super::*;

    /// Generate hit objects from beat times (auto-mapping)
    pub fn generate_from_beats(
        beats: &[f64],
        pattern: PatternType,
        approach_time: f64,
    ) -> Vec<HitObject> {
        let mut objects = Vec::new();
        let pattern_positions = pattern.positions();

        for (i, &time) in beats.iter().enumerate() {
            let pos_idx = i % pattern_positions.len();
            let (x, y) = pattern_positions[pos_idx];

            objects.push(HitObject::new_circle(time, x, y));
        }

        objects
    }

    /// Pattern types for auto-generation
    #[derive(Debug, Clone, Copy)]
    pub enum PatternType {
        Random,
        Circle,
        Grid,
        Stream,
        Jump,
    }

    impl PatternType {
        /// Get positions for this pattern type (normalized 0.0-1.0)
        pub fn positions(&self) -> Vec<(f32, f32)> {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            match self {
                PatternType::Random => {
                    // Generate 16 random positions
                    (0..16)
                        .map(|_| (rng.gen_range(0.1..0.9), rng.gen_range(0.1..0.9)))
                        .collect()
                }
                PatternType::Circle => {
                    // Circle pattern with 8 positions
                    (0..8)
                        .map(|i| {
                            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                            let x = 0.5 + 0.3 * angle.cos();
                            let y = 0.5 + 0.3 * angle.sin();
                            (x, y)
                        })
                        .collect()
                }
                PatternType::Grid => {
                    // 4x4 grid
                    let mut positions = Vec::new();
                    for row in 0..4 {
                        for col in 0..4 {
                            let x = 0.2 + (col as f32 / 3.0) * 0.6;
                            let y = 0.2 + (row as f32 / 3.0) * 0.6;
                            positions.push((x, y));
                        }
                    }
                    positions
                }
                PatternType::Stream => {
                    // Alternating left-right stream
                    vec![
                        (0.3, 0.5),
                        (0.7, 0.5),
                        (0.3, 0.4),
                        (0.7, 0.6),
                        (0.3, 0.6),
                        (0.7, 0.4),
                        (0.5, 0.3),
                        (0.5, 0.7),
                    ]
                }
                PatternType::Jump => {
                    // Large jumps across screen
                    vec![(0.2, 0.2), (0.8, 0.8), (0.8, 0.2), (0.2, 0.8)]
                }
            }
        }
    }
}
