// src/beatmap.rs

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Unique identifier for hit objects
pub type HitObjectId = u64;

/// Beatmap file format version
pub const BEATMAP_FORMAT_VERSION: u32 = 1;

/// A complete beatmap containing all metadata, timing, and hit objects
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Beatmap {
    /// Format version for migration support
    pub version: u32,
    /// Beatmap metadata
    pub metadata: BeatmapMetadata,
    /// Timing points defining BPM and signature changes
    pub timing_points: Vec<TimingPoint>,
    /// All hit objects in the beatmap
    pub hit_objects: Vec<HitObject>,
    /// Beatmap settings
    pub settings: BeatmapSettings,
    /// Bookmarks for quick navigation
    pub bookmarks: Vec<Bookmark>,
    /// Background image path
    pub background_path: Option<String>,
    /// Audio file path (relative to beatmap)
    pub audio_path: String,
    /// Preview point in seconds
    pub preview_time: f64,
    /// Tags for searching/categorization
    pub tags: Vec<String>,
}

impl Default for Beatmap {
    fn default() -> Self {
        Self {
            version: BEATMAP_FORMAT_VERSION,
            metadata: BeatmapMetadata::default(),
            timing_points: vec![TimingPoint::default()],
            hit_objects: Vec::new(),
            settings: BeatmapSettings::default(),
            bookmarks: Vec::new(),
            background_path: None,
            audio_path: String::new(),
            preview_time: 0.0,
            tags: Vec::new(),
        }
    }
}

impl Beatmap {
    /// Create a new empty beatmap
    pub fn new(title: String, artist: String, audio_path: String) -> Self {
        Self {
            metadata: BeatmapMetadata {
                title,
                artist,
                ..Default::default()
            },
            audio_path,
            ..Default::default()
        }
    }

    /// Save beatmap to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        match serde_json::to_string_pretty(self) {
            Ok(json) => match fs::write(path, json) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to write beatmap: {}", e)),
            },
            Err(e) => Err(format!("Failed to serialize beatmap: {}", e)),
        }
    }

    /// Load beatmap from a JSON file
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        if !Path::new(path).exists() {
            return Err(format!("Beatmap file not found: {}", path));
        }

        match fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str::<'_, Beatmap>(&contents) {
                Ok(mut beatmap) => {
                    // Ensure version is set
                    if beatmap.version == 0 {
                        beatmap.version = BEATMAP_FORMAT_VERSION;
                    }
                    // Sort hit objects by time
                    beatmap.sort_hit_objects();
                    Ok(beatmap)
                }
                Err(e) => Err(format!("Failed to parse beatmap: {}", e)),
            },
            Err(e) => Err(format!("Failed to read beatmap: {}", e)),
        }
    }

    /// Add a hit object
    pub fn add_hit_object(&mut self, object: HitObject) {
        self.hit_objects.push(object);
        self.sort_hit_objects();
    }

    /// Remove a hit object by ID
    pub fn remove_hit_object(&mut self, id: HitObjectId) -> Option<HitObject> {
        if let Some(index) = self.hit_objects.iter().position(|h| h.id == id) {
            Some(self.hit_objects.remove(index))
        } else {
            None
        }
    }

    /// Sort hit objects by time
    pub fn sort_hit_objects(&mut self) {
        self.hit_objects.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get BPM at a specific time
    pub fn get_bpm_at(&self, time: f64) -> f64 {
        self.timing_points
            .iter()
            .rev()
            .find(|tp| tp.time <= time)
            .map(|tp| tp.bpm)
            .unwrap_or(120.0)
    }

    /// Get beat length at a specific time (in seconds)
    pub fn get_beat_length_at(&self, time: f64) -> f64 {
        let bpm = self.get_bpm_at(time);
        60.0 / bpm
    }

    /// Convert time to beat number
    pub fn time_to_beat(&self, time: f64) -> f64 {
        let mut beat = 0.0;
        let mut last_time = 0.0;
        let mut last_bpm = 120.0;

        for tp in &self.timing_points {
            if tp.time > time {
                break;
            }
            beat += (tp.time - last_time) / (60.0 / last_bpm);
            last_time = tp.time;
            last_bpm = tp.bpm;
        }

        beat += (time - last_time) / (60.0 / last_bpm);
        beat
    }

    /// Convert beat number to time
    pub fn beat_to_time(&self, beat: f64) -> f64 {
        let mut time = 0.0;
        let mut last_beat = 0.0;
        let mut last_bpm = 120.0;

        for tp in &self.timing_points {
            let tp_beat = last_beat + (tp.time - time) / (60.0 / last_bpm);
            if tp_beat > beat {
                break;
            }
            time = tp.time;
            last_beat = tp_beat;
            last_bpm = tp.bpm;
        }

        time + (beat - last_beat) * (60.0 / last_bpm)
    }

    /// Snap time to nearest beat divisor
    pub fn snap_time(&self, time: f64, divisor: u32) -> f64 {
        let beat = self.time_to_beat(time);
        let snapped_beat = (beat * divisor as f64).round() / divisor as f64;
        self.beat_to_time(snapped_beat)
    }

    /// Get hit objects in a time range
    pub fn get_hit_objects_in_range(&self, start: f64, end: f64) -> Vec<&HitObject> {
        self.hit_objects
            .iter()
            .filter(|h| h.time >= start && h.time <= end)
            .collect()
    }

    /// Get object count statistics
    pub fn get_object_stats(&self) -> ObjectStats {
        let mut stats = ObjectStats::default();
        for obj in &self.hit_objects {
            match obj.kind {
                HitObjectKind::Circle => stats.circles += 1,
                HitObjectKind::Slider { .. } => stats.sliders += 1,
                HitObjectKind::Spinner { .. } => stats.spinners += 1,
            }
        }
        stats.total = self.hit_objects.len();
        stats
    }

    /// Calculate total duration
    pub fn get_duration(&self) -> f64 {
        self.hit_objects.last().map(|h| h.time).unwrap_or(0.0)
    }

    /// Generate a unique ID for new hit objects
    pub fn generate_hit_object_id(&self) -> HitObjectId {
        self.hit_objects.iter().map(|h| h.id).max().unwrap_or(0) + 1
    }
}

/// Beatmap metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapMetadata {
    /// Song title
    pub title: String,
    /// Song artist
    pub artist: String,
    /// Beatmap creator/mapper
    pub creator: String,
    /// Beatmap version/difficulty name
    pub version: String,
    /// Source (anime, game, etc.)
    pub source: Option<String>,
    /// Beatmap ID (for online systems)
    pub beatmap_id: Option<u64>,
    /// Set ID (for online systems)
    pub set_id: Option<u64>,
}

impl Default for BeatmapMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            creator: String::new(),
            version: "Normal".to_string(),
            source: None,
            beatmap_id: None,
            set_id: None,
        }
    }
}

/// Timing point defining BPM and time signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingPoint {
    /// Time in seconds when this timing point takes effect
    pub time: f64,
    /// BPM (beats per minute)
    pub bpm: f64,
    /// Time signature numerator (beats per measure)
    pub meter: u32,
    /// Whether this is an inherited (uninherited) timing point
    pub inherited: bool,
    /// Volume percentage (0-100)
    pub volume: u32,
    /// Kiai mode (special section with effects)
    pub kiai: bool,
}

impl Default for TimingPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            bpm: 120.0,
            meter: 4,
            inherited: false,
            volume: 100,
            kiai: false,
        }
    }
}

/// A single hit object in the beatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitObject {
    /// Unique identifier
    pub id: HitObjectId,
    /// Time in seconds when the object should be hit
    pub time: f64,
    /// Position on screen
    pub position: Vec2,
    /// Type of hit object
    pub kind: HitObjectKind,
    /// New combo indicator
    pub new_combo: bool,
    /// Combo color index
    pub combo_index: u32,
    /// Hitsound addition
    pub hitsound: Hitsound,
    /// Custom sample set
    pub sample_set: Option<SampleSet>,
}

/// Type of hit object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitObjectKind {
    /// Standard circle
    Circle,
    /// Slider with control points
    Slider {
        /// Control points defining the slider path
        control_points: Vec<Vec2>,
        /// Number of repeats
        repeats: u32,
        /// Length in osu! pixels
        pixel_length: f64,
        /// Slider velocity multiplier
        velocity: f64,
    },
    /// Spinner (hold for duration)
    Spinner {
        /// End time of the spinner
        end_time: f64,
    },
}

/// Hitsound types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Hitsound {
    #[default]
    Normal,
    Whistle,
    Finish,
    Clap,
}

/// Sample set for hitsounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleSet {
    pub normal_set: u32,
    pub addition_set: u32,
    pub index: u32,
    pub volume: u32,
    pub filename: Option<String>,
}

/// Beatmap difficulty settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapSettings {
    /// Circle size (CS) - affects circle radius
    pub circle_size: f32,
    /// Approach rate (AR) - affects approach speed
    pub approach_rate: f32,
    /// Overall difficulty (OD) - affects timing windows
    pub overall_difficulty: f32,
    /// HP drain rate - affects health drain
    pub hp_drain: f32,
    /// Slider multiplier - affects slider speed
    pub slider_multiplier: f64,
    /// Slider tick rate
    pub slider_tick_rate: f64,
    /// Stack leniency
    pub stack_leniency: f32,
}

impl Default for BeatmapSettings {
    fn default() -> Self {
        Self {
            circle_size: 4.0,
            approach_rate: 9.0,
            overall_difficulty: 8.0,
            hp_drain: 5.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,
            stack_leniency: 0.7,
        }
    }
}

impl BeatmapSettings {
    /// Get circle radius in pixels based on circle size
    pub fn get_circle_radius(&self) -> f32 {
        // CS 5 = 50px radius, lower CS = larger circles
        54.4 - 4.48 * self.circle_size
    }

    /// Get approach time in seconds based on approach rate
    pub fn get_approach_time(&self) -> f64 {
        // AR 5 = 1.2s approach time, higher AR = faster approach
        if self.approach_rate < 5.0 {
            1.8 - (self.approach_rate as f64 - 5.0) * 0.15
        } else {
            1.2 - (self.approach_rate as f64 - 5.0) * 0.075
        }
    }

    /// Get timing windows for different hit judgments (in seconds)
    pub fn get_timing_windows(&self) -> TimingWindows {
        let od = self.overall_difficulty as f64;
        TimingWindows {
            perfect: 0.08 - 0.006 * od,
            good: 0.14 - 0.008 * od,
            okay: 0.2 - 0.01 * od,
        }
    }
}

/// Timing windows for hit judgments
#[derive(Debug, Clone, Copy)]
pub struct TimingWindows {
    pub perfect: f64,
    pub good: f64,
    pub okay: f64,
}

/// Bookmark for quick navigation in editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub time: f64,
    pub name: Option<String>,
    pub color: Option<String>,
}

/// Object count statistics
#[derive(Debug, Clone, Default)]
pub struct ObjectStats {
    pub total: usize,
    pub circles: usize,
    pub sliders: usize,
    pub spinners: usize,
}

/// Asset manager for beatmaps
#[derive(Debug, Clone, Resource)]
pub struct BeatmapAssets {
    /// Loaded beatmaps by path
    pub beatmaps: HashMap<String, Beatmap>,
    /// Current beatmap being played/edited
    pub current_beatmap: Option<String>,
    /// Beatmaps directory
    pub beatmaps_dir: String,
}

impl Default for BeatmapAssets {
    fn default() -> Self {
        Self {
            beatmaps: HashMap::new(),
            current_beatmap: None,
            beatmaps_dir: "src/assets/beatmaps".to_string(),
        }
    }
}

impl BeatmapAssets {
    /// Create new beatmap assets manager
    pub fn new(beatmaps_dir: String) -> Self {
        Self {
            beatmaps: HashMap::new(),
            current_beatmap: None,
            beatmaps_dir,
        }
    }

    /// Load all beatmaps from the beatmaps directory
    pub fn load_all(&mut self) -> Result<usize, String> {
        self.beatmaps.clear();

        let path = Path::new(&self.beatmaps_dir);
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create beatmaps dir: {}", e))?;
            return Ok(0);
        }

        let mut count = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    let path_str = path.to_string_lossy().to_string();
                    if let Ok(beatmap) = Beatmap::load_from_file(&path_str) {
                        self.beatmaps.insert(path_str, beatmap);
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get a beatmap by path
    pub fn get(&self, path: &str) -> Option<&Beatmap> {
        self.beatmaps.get(path)
    }

    /// Get mutable reference to a beatmap
    pub fn get_mut(&mut self, path: &str) -> Option<&mut Beatmap> {
        self.beatmaps.get_mut(path)
    }

    /// Get current beatmap
    pub fn current(&self) -> Option<&Beatmap> {
        self.current_beatmap
            .as_ref()
            .and_then(|p| self.beatmaps.get(p))
    }

    /// Get mutable current beatmap
    pub fn current_mut(&mut self) -> Option<&mut Beatmap> {
        self.current_beatmap
            .as_ref()
            .and_then(|p| self.beatmaps.get_mut(p))
    }

    /// Set current beatmap
    pub fn set_current(&mut self, path: Option<String>) {
        self.current_beatmap = path;
    }

    /// Add or update a beatmap
    pub fn add(&mut self, path: String, beatmap: Beatmap) {
        self.beatmaps.insert(path, beatmap);
    }

    /// Save a beatmap to file
    pub fn save(&self, path: &str) -> Result<(), String> {
        if let Some(beatmap) = self.beatmaps.get(path) {
            beatmap.save_to_file(path)?;
            Ok(())
        } else {
            Err("Beatmap not found".to_string())
        }
    }

    /// Get all beatmap paths
    pub fn get_all_paths(&self) -> Vec<&String> {
        self.beatmaps.keys().collect()
    }

    /// Search beatmaps by title, artist, or tags
    pub fn search(&self, query: &str) -> Vec<&Beatmap> {
        let query = query.to_lowercase();
        self.beatmaps
            .values()
            .filter(|b| {
                b.metadata.title.to_lowercase().contains(&query)
                    || b.metadata.artist.to_lowercase().contains(&query)
                    || b.metadata.creator.to_lowercase().contains(&query)
                    || b.tags.iter().any(|t| t.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Get beatmaps sorted by creation time (newest first)
    pub fn get_recent(&self, limit: usize) -> Vec<&Beatmap> {
        let mut beatmaps: Vec<_> = self.beatmaps.values().collect();
        // For now, just return in arbitrary order
        // In production, would sort by file modification time
        beatmaps.into_iter().take(limit).collect()
    }
}

/// Beatmap list entry for UI display
#[derive(Debug, Clone)]
pub struct BeatmapListEntry {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub version: String,
    pub object_count: usize,
    pub duration: f64,
}

impl From<&Beatmap> for BeatmapListEntry {
    fn from(beatmap: &Beatmap) -> Self {
        Self {
            path: String::new(), // Must be set separately
            title: beatmap.metadata.title.clone(),
            artist: beatmap.metadata.artist.clone(),
            creator: beatmap.metadata.creator.clone(),
            version: beatmap.metadata.version.clone(),
            object_count: beatmap.hit_objects.len(),
            duration: beatmap.get_duration(),
        }
    }
}

/// Editor tool types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Select,
    Circle,
    Slider,
    Spinner,
    Delete,
}

impl EditorTool {
    pub fn display_name(&self) -> &'static str {
        match self {
            EditorTool::Select => "Select",
            EditorTool::Circle => "Circle",
            EditorTool::Slider => "Slider",
            EditorTool::Spinner => "Spinner",
            EditorTool::Delete => "Delete",
        }
    }

    pub fn all() -> Vec<EditorTool> {
        vec![
            EditorTool::Select,
            EditorTool::Circle,
            EditorTool::Slider,
            EditorTool::Spinner,
            EditorTool::Delete,
        ]
    }
}

/// Beat snap divisors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatDivisor {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Six = 6,
    Eight = 8,
    Twelve = 12,
    Sixteen = 16,
}

impl BeatDivisor {
    pub fn value(&self) -> u32 {
        *self as u32
    }

    pub fn all() -> Vec<BeatDivisor> {
        vec![
            BeatDivisor::One,
            BeatDivisor::Two,
            BeatDivisor::Three,
            BeatDivisor::Four,
            BeatDivisor::Six,
            BeatDivisor::Eight,
            BeatDivisor::Twelve,
            BeatDivisor::Sixteen,
        ]
    }

    pub fn display_name(&self) -> String {
        format!("1/{}", self.value())
    }
}

impl Default for BeatDivisor {
    fn default() -> Self {
        BeatDivisor::Four
    }
}
