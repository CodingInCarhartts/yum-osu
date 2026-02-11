// src/editor.rs

use crate::beatmap::{
    BeatDivisor, Beatmap, BeatmapAssets, BeatmapSettings, EditorTool, HitObject, HitObjectId,
    HitObjectKind, Hitsound, TimingPoint,
};
use crate::constants::*;
use crate::structs::GameAssets;
use crate::ui::UiElement;
use bevy::prelude::*;
use std::time::Instant;

/// Editor state resource
#[derive(Debug, Clone, Resource)]
pub struct EditorState {
    /// Current tool
    pub current_tool: EditorTool,
    /// Current beat divisor
    pub beat_divisor: BeatDivisor,
    /// Current time in the song (seconds)
    pub current_time: f64,
    /// Audio playback speed
    pub playback_speed: f32,
    /// Is audio playing
    pub is_playing: bool,
    /// Timeline zoom level (pixels per second)
    pub timeline_zoom: f32,
    /// Selected object IDs
    pub selected_objects: Vec<HitObjectId>,
    /// Playback start time (for calculating current time)
    pub playback_start: Option<Instant>,
    /// Time offset at playback start
    pub playback_start_time: f64,
    /// Show grid
    pub show_grid: bool,
    /// Show waveform
    pub show_waveform: bool,
    /// Snap to grid
    pub snap_enabled: bool,
    /// Grid size
    pub grid_size: f32,
    /// New combo mode
    pub new_combo_mode: bool,
    /// Current hitsound
    pub current_hitsound: Hitsound,
    /// Clipboard for copy/paste
    pub clipboard: Vec<HitObject>,
    /// Undo history
    pub undo_stack: Vec<EditorAction>,
    /// Redo history
    pub redo_stack: Vec<EditorAction>,
    /// Maximum undo history size
    pub max_undo_size: usize,
    /// Is in editor test mode
    pub test_mode: bool,
    /// Timeline scroll offset
    pub timeline_scroll: f32,
    /// Playfield zoom
    pub playfield_zoom: f32,
    /// Current beatmap path
    pub current_beatmap_path: Option<String>,
    /// Bookmarks panel visible
    pub show_bookmarks: bool,
    /// Timing panel visible
    pub show_timing: bool,
    /// Settings panel visible
    pub show_settings: bool,
    /// Audio file duration (if known)
    pub audio_duration: Option<f64>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_tool: EditorTool::Select,
            beat_divisor: BeatDivisor::Four,
            current_time: 0.0,
            playback_speed: 1.0,
            is_playing: false,
            timeline_zoom: 100.0,
            selected_objects: Vec::new(),
            playback_start: None,
            playback_start_time: 0.0,
            show_grid: true,
            show_waveform: true,
            snap_enabled: true,
            grid_size: 32.0,
            new_combo_mode: false,
            current_hitsound: Hitsound::Normal,
            clipboard: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_size: 50,
            test_mode: false,
            timeline_scroll: 0.0,
            playfield_zoom: 1.0,
            current_beatmap_path: None,
            show_bookmarks: false,
            show_timing: false,
            show_settings: false,
            audio_duration: None,
        }
    }
}

impl EditorState {
    /// Create new editor state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle playback
    pub fn toggle_playback(&mut self) {
        if self.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Start playback
    pub fn play(&mut self) {
        if !self.is_playing {
            self.is_playing = true;
            self.playback_start = Some(Instant::now());
            self.playback_start_time = self.current_time;
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.is_playing {
            // Update current time before pausing
            self.update_current_time();
            self.is_playing = false;
            self.playback_start = None;
        }
    }

    /// Stop playback and return to start
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.playback_start = None;
        self.current_time = 0.0;
    }

    /// Update current time based on playback
    pub fn update_current_time(&mut self) {
        if let Some(start) = self.playback_start {
            let elapsed = start.elapsed().as_secs_f64() * self.playback_speed as f64;
            self.current_time = self.playback_start_time + elapsed;
        }
    }

    /// Seek to a specific time
    pub fn seek_to(&mut self, time: f64) {
        self.current_time = time.max(0.0);
        if self.is_playing {
            self.playback_start = Some(Instant::now());
            self.playback_start_time = self.current_time;
        }
    }

    /// Seek forward by a beat
    pub fn seek_forward(&mut self, beatmap: &Beatmap) {
        let beat_length = beatmap.get_beat_length_at(self.current_time);
        self.seek_to(self.current_time + beat_length);
    }

    /// Seek backward by a beat
    pub fn seek_backward(&mut self, beatmap: &Beatmap) {
        let beat_length = beatmap.get_beat_length_at(self.current_time);
        self.seek_to(self.current_time - beat_length);
    }

    /// Select an object
    pub fn select_object(&mut self, id: HitObjectId, add_to_selection: bool) {
        if add_to_selection {
            if !self.selected_objects.contains(&id) {
                self.selected_objects.push(id);
            }
        } else {
            self.selected_objects.clear();
            self.selected_objects.push(id);
        }
    }

    /// Deselect all objects
    pub fn deselect_all(&mut self) {
        self.selected_objects.clear();
    }

    /// Delete selected objects and return the action for undo
    pub fn delete_selected(&mut self, beatmap: &mut Beatmap) -> Option<EditorAction> {
        if self.selected_objects.is_empty() {
            return None;
        }

        let mut deleted = Vec::new();
        for id in &self.selected_objects {
            if let Some(obj) = beatmap.remove_hit_object(*id) {
                deleted.push(obj);
            }
        }
        self.selected_objects.clear();

        if deleted.is_empty() {
            None
        } else {
            Some(EditorAction::DeleteObjects { objects: deleted })
        }
    }

    /// Add an object and return the action for undo
    pub fn add_object(&mut self, beatmap: &mut Beatmap, position: Vec2) -> Option<EditorAction> {
        let time = if self.snap_enabled {
            beatmap.snap_time(self.current_time, self.beat_divisor.value())
        } else {
            self.current_time
        };

        let id = beatmap.generate_hit_object_id();
        let kind = match self.current_tool {
            EditorTool::Circle => HitObjectKind::Circle,
            EditorTool::Slider => HitObjectKind::Slider {
                control_points: vec![position, position + Vec2::new(100.0, 0.0)],
                repeats: 0,
                pixel_length: 100.0,
                velocity: 1.0,
            },
            EditorTool::Spinner => HitObjectKind::Spinner {
                end_time: time + 1.0,
            },
            _ => return None,
        };

        let object = HitObject {
            id,
            time,
            position,
            kind,
            new_combo: self.new_combo_mode,
            combo_index: 0,
            hitsound: self.current_hitsound,
            sample_set: None,
        };

        beatmap.add_hit_object(object.clone());
        self.select_object(id, false);

        Some(EditorAction::AddObject { object })
    }

    /// Record an action for undo
    pub fn record_action(&mut self, action: EditorAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_undo_size {
            self.undo_stack.remove(0);
        }
        // Clear redo stack on new action
        self.redo_stack.clear();
    }

    /// Undo last action
    pub fn undo(&mut self, beatmap: &mut Beatmap) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            let inverse = action.undo(beatmap);
            self.redo_stack.push(inverse);
            true
        } else {
            false
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self, beatmap: &mut Beatmap) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            let inverse = action.undo(beatmap);
            self.undo_stack.push(inverse);
            true
        } else {
            false
        }
    }

    /// Get selected objects from beatmap
    pub fn get_selected_objects(&self, beatmap: &Beatmap) -> Vec<&HitObject> {
        beatmap
            .hit_objects
            .iter()
            .filter(|obj| self.selected_objects.contains(&obj.id))
            .collect()
    }

    /// Copy selected objects to clipboard
    pub fn copy_selected(&mut self, beatmap: &Beatmap) {
        self.clipboard = beatmap
            .hit_objects
            .iter()
            .filter(|obj| self.selected_objects.contains(&obj.id))
            .cloned()
            .collect();
    }

    /// Paste objects from clipboard
    pub fn paste(&mut self, beatmap: &mut Beatmap) -> Vec<EditorAction> {
        let mut actions = Vec::new();
        let time_offset = self.current_time;
        let mut new_selection = Vec::new();

        for obj in &self.clipboard {
            let id = beatmap.generate_hit_object_id();
            let new_obj = HitObject {
                id,
                time: obj.time + time_offset,
                position: obj.position,
                kind: obj.kind.clone(),
                new_combo: obj.new_combo,
                combo_index: obj.combo_index,
                hitsound: obj.hitsound,
                sample_set: obj.sample_set.clone(),
            };
            beatmap.add_hit_object(new_obj.clone());
            new_selection.push(id);
            actions.push(EditorAction::AddObject { object: new_obj });
        }

        self.selected_objects = new_selection;
        actions
    }

    /// Set tool
    pub fn set_tool(&mut self, tool: EditorTool) {
        self.current_tool = tool;
        // Clear selection when switching tools (except select)
        if tool != EditorTool::Select {
            self.selected_objects.clear();
        }
    }

    /// Toggle grid snapping
    pub fn toggle_snap(&mut self) {
        self.snap_enabled = !self.snap_enabled;
    }

    /// Get the object under a position at the current time
    pub fn get_object_at_position(
        &self,
        beatmap: &Beatmap,
        position: Vec2,
        tolerance: f32,
    ) -> Option<HitObjectId> {
        beatmap
            .hit_objects
            .iter()
            .find(|obj| {
                let time_diff = (obj.time - self.current_time).abs();
                if time_diff > 0.1 {
                    return false;
                }
                obj.position.distance(position) < tolerance
            })
            .map(|obj| obj.id)
    }
}

/// Editor actions for undo/redo
#[derive(Debug, Clone)]
pub enum EditorAction {
    AddObject {
        object: HitObject,
    },
    DeleteObjects {
        objects: Vec<HitObject>,
    },
    MoveObjects {
        moves: Vec<ObjectMove>,
    },
    ModifyTiming {
        old_points: Vec<TimingPoint>,
        new_points: Vec<TimingPoint>,
    },
    ModifySettings {
        old_settings: BeatmapSettings,
        new_settings: BeatmapSettings,
    },
}

/// Object move data for undo
#[derive(Debug, Clone)]
pub struct ObjectMove {
    pub id: HitObjectId,
    pub old_position: Vec2,
    pub new_position: Vec2,
    pub old_time: f64,
    pub new_time: f64,
}

impl EditorAction {
    /// Undo the action and return the inverse action
    pub fn undo(self, beatmap: &mut Beatmap) -> EditorAction {
        match self {
            EditorAction::AddObject { object } => {
                beatmap.remove_hit_object(object.id);
                EditorAction::DeleteObjects {
                    objects: vec![object],
                }
            }
            EditorAction::DeleteObjects { objects } => {
                for obj in &objects {
                    beatmap.add_hit_object(obj.clone());
                }
                EditorAction::AddObject {
                    object: objects.into_iter().next().unwrap(),
                }
            }
            EditorAction::MoveObjects { moves } => {
                let inverse_moves: Vec<_> = moves
                    .iter()
                    .map(|m| {
                        if let Some(obj) = beatmap.hit_objects.iter_mut().find(|o| o.id == m.id) {
                            obj.position = m.old_position;
                            obj.time = m.old_time;
                        }
                        ObjectMove {
                            id: m.id,
                            old_position: m.new_position,
                            new_position: m.old_position,
                            old_time: m.new_time,
                            new_time: m.old_time,
                        }
                    })
                    .collect();
                EditorAction::MoveObjects {
                    moves: inverse_moves,
                }
            }
            EditorAction::ModifyTiming { old_points, .. } => {
                let current = beatmap.timing_points.clone();
                beatmap.timing_points = old_points.clone();
                EditorAction::ModifyTiming {
                    old_points: current,
                    new_points: old_points,
                }
            }
            EditorAction::ModifySettings { old_settings, .. } => {
                let current = beatmap.settings.clone();
                beatmap.settings = old_settings.clone();
                EditorAction::ModifySettings {
                    old_settings: current,
                    new_settings: old_settings,
                }
            }
        }
    }
}

/// Editor UI state
#[derive(Debug, Clone, Resource)]
pub struct EditorUIState {
    /// Left panel width
    pub left_panel_width: f32,
    /// Right panel width
    pub right_panel_width: f32,
    /// Timeline height
    pub timeline_height: f32,
    /// Toolbar height
    pub toolbar_height: f32,
    /// Is left panel visible
    pub left_panel_visible: bool,
    /// Is right panel visible
    pub right_panel_visible: bool,
    /// Selected tab in left panel
    pub left_panel_tab: EditorLeftTab,
    /// Selected tab in right panel
    pub right_panel_tab: EditorRightTab,
    /// Hover info
    pub hover_info: Option<String>,
    /// Status message
    pub status_message: Option<(String, Instant)>,
}

impl Default for EditorUIState {
    fn default() -> Self {
        Self {
            left_panel_width: 250.0,
            right_panel_width: 280.0,
            timeline_height: 150.0,
            toolbar_height: 50.0,
            left_panel_visible: true,
            right_panel_visible: true,
            left_panel_tab: EditorLeftTab::Tools,
            right_panel_tab: EditorRightTab::Properties,
            hover_info: None,
            status_message: None,
        }
    }
}

impl EditorUIState {
    /// Show a status message
    pub fn show_status(&mut self, message: String, duration_secs: u64) {
        self.status_message = Some((message, Instant::now()));
    }

    /// Check if status message has expired
    pub fn update_status(&mut self, duration_secs: u64) {
        if let Some((_, start)) = &self.status_message {
            if start.elapsed().as_secs() > duration_secs {
                self.status_message = None;
            }
        }
    }
}

/// Left panel tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorLeftTab {
    Tools,
    Timing,
    Bookmarks,
}

/// Right panel tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorRightTab {
    Properties,
    Settings,
    Metadata,
}

/// Grid constants
pub const GRID_COLUMNS: usize = 16;
pub const GRID_ROWS: usize = 12;
pub const PLAYFIELD_WIDTH: f32 = 640.0;
pub const PLAYFIELD_HEIGHT: f32 = 480.0;

/// Calculate grid position from screen coordinates
pub fn screen_to_grid(screen_pos: Vec2, window_size: Vec2, zoom: f32, pan: Vec2) -> Vec2 {
    let center = Vec2::new(window_size.x / 2.0, window_size.y / 2.0);
    (screen_pos - center - pan) / zoom
}

/// Calculate screen position from grid coordinates
pub fn grid_to_screen(grid_pos: Vec2, window_size: Vec2, zoom: f32, pan: Vec2) -> Vec2 {
    let center = Vec2::new(window_size.x / 2.0, window_size.y / 2.0);
    grid_pos * zoom + center + pan
}

/// Snap position to grid
pub fn snap_to_grid(position: Vec2, grid_size: f32) -> Vec2 {
    Vec2::new(
        (position.x / grid_size).round() * grid_size,
        (position.y / grid_size).round() * grid_size,
    )
}

/// Timeline rendering constants
pub const TIMELINE_BEAT_HEIGHT: f32 = 20.0;
pub const TIMELINE_OBJECT_HEIGHT: f32 = 16.0;
pub const TIMELINE_WAVEFORM_HEIGHT: f32 = 60.0;

/// Get beat line opacity based on beat importance
pub fn get_beat_line_opacity(beat_index: usize) -> f32 {
    if beat_index % 16 == 0 {
        1.0 // Measure line
    } else if beat_index % 4 == 0 {
        0.7 // Beat line
    } else {
        0.3 // Sub-beat line
    }
}

/// Calculate timeline position for a given time
pub fn time_to_timeline_pos(time: f64, zoom: f32, scroll: f32) -> f32 {
    time as f32 * zoom + scroll
}

/// Calculate time from timeline position
pub fn timeline_pos_to_time(pos: f32, zoom: f32, scroll: f32) -> f64 {
    ((pos - scroll) / zoom) as f64
}
