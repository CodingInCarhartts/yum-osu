// src/editor_input.rs

use crate::beatmap::{BeatDivisor, BeatmapAssets, EditorTool};
use crate::constants::*;
use crate::editor::{
    screen_to_grid, snap_to_grid, EditorAction, EditorLeftTab, EditorRightTab, EditorState,
    EditorUIState,
};
use crate::editor_ui::*;
use bevy::prelude::*;
use bevy::window::Window;

/// Handle editor input
pub fn handle_editor_input(
    mut editor_state: ResMut<EditorState>,
    mut editor_ui: ResMut<EditorUIState>,
    mut beatmap_assets: ResMut<BeatmapAssets>,
    mut next_state: ResMut<NextState<crate::AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    let window = windows.single();

    // Update playback time
    if editor_state.is_playing {
        editor_state.update_current_time();
    }

    // ESC to exit editor
    if keyboard.just_pressed(KeyCode::Escape) {
        // Save current beatmap before exiting
        if let Some(path) = &editor_state.current_beatmap_path {
            if let Err(e) = beatmap_assets.save(path) {
                eprintln!("Failed to save beatmap: {}", e);
            }
        }
        next_state.set(crate::AppState::Menu);
        return;
    }

    // Playback controls
    if keyboard.just_pressed(KeyCode::Space) {
        editor_state.toggle_playback();
    }

    if keyboard.just_pressed(KeyCode::Comma) {
        editor_state.seek_backward(
            beatmap_assets
                .current()
                .unwrap_or(&crate::beatmap::Beatmap::default()),
        );
    }

    if keyboard.just_pressed(KeyCode::Period) {
        editor_state.seek_forward(
            beatmap_assets
                .current()
                .unwrap_or(&crate::beatmap::Beatmap::default()),
        );
    }

    // Tool shortcuts
    if keyboard.just_pressed(KeyCode::Digit1) {
        editor_state.set_tool(EditorTool::Select);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        editor_state.set_tool(EditorTool::Circle);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        editor_state.set_tool(EditorTool::Slider);
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        editor_state.set_tool(EditorTool::Spinner);
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        editor_state.set_tool(EditorTool::Delete);
    }

    // Snap toggle
    if keyboard.just_pressed(KeyCode::KeyY) {
        editor_state.toggle_snap();
    }

    // Grid toggle
    if keyboard.just_pressed(KeyCode::KeyG) {
        editor_state.show_grid = !editor_state.show_grid;
    }

    // New combo toggle
    if keyboard.just_pressed(KeyCode::KeyQ) {
        editor_state.new_combo_mode = !editor_state.new_combo_mode;
    }

    // Undo/Redo
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyZ) {
            if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                // Redo
                if let Some(beatmap) = beatmap_assets.current_mut() {
                    editor_state.redo(beatmap);
                }
            } else {
                // Undo
                if let Some(beatmap) = beatmap_assets.current_mut() {
                    editor_state.undo(beatmap);
                }
            }
        }
    }

    // Copy/Paste
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyC) {
            if let Some(beatmap) = beatmap_assets.current() {
                editor_state.copy_selected(beatmap);
            }
        }
        if keyboard.just_pressed(KeyCode::KeyV) {
            if let Some(beatmap) = beatmap_assets.current_mut() {
                let actions = editor_state.paste(beatmap);
                for action in actions {
                    editor_state.record_action(action);
                }
            }
        }
    }

    // Delete selected
    if keyboard.just_pressed(KeyCode::Delete) {
        if let Some(beatmap) = beatmap_assets.current_mut() {
            if let Some(action) = editor_state.delete_selected(beatmap) {
                editor_state.record_action(action);
            }
        }
    }

    // Beat divisor shortcuts
    if keyboard.just_pressed(KeyCode::KeyA) {
        editor_state.beat_divisor = BeatDivisor::One;
    }
    if keyboard.just_pressed(KeyCode::KeyS) {
        editor_state.beat_divisor = BeatDivisor::Two;
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        editor_state.beat_divisor = BeatDivisor::Four;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        editor_state.beat_divisor = BeatDivisor::Eight;
    }
    if keyboard.just_pressed(KeyCode::KeyX) {
        editor_state.beat_divisor = BeatDivisor::Three;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        editor_state.beat_divisor = BeatDivisor::Six;
    }

    // Zoom controls
    if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        editor_state.timeline_zoom *= 1.05;
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        editor_state.timeline_zoom *= 0.95;
    }

    // Mouse input handling
    if let Some(cursor_pos) = window.cursor_position() {
        let screen_w = window.width();
        let screen_h = window.height();
        let world_x = cursor_pos.x - screen_w / 2.0;
        let world_y = screen_h / 2.0 - cursor_pos.y;

        // Check if clicking on UI elements
        let in_toolbar = world_y > screen_h / 2.0 - editor_ui.toolbar_height;
        let in_timeline = world_y < -screen_h / 2.0 + editor_ui.timeline_height + 20.0;
        let in_left_panel =
            editor_ui.left_panel_visible && world_x < -screen_w / 2.0 + editor_ui.left_panel_width;
        let in_right_panel =
            editor_ui.right_panel_visible && world_x > screen_w / 2.0 - editor_ui.right_panel_width;

        let in_playfield = !in_toolbar && !in_timeline && !in_left_panel && !in_right_panel;

        // Handle left click
        if mouse_input.just_pressed(MouseButton::Left) {
            if in_playfield {
                handle_playfield_click(
                    &mut editor_state,
                    beatmap_assets.as_mut(),
                    world_x,
                    world_y,
                );
            } else if in_timeline {
                handle_timeline_click(
                    &mut editor_state,
                    &editor_ui,
                    beatmap_assets.current(),
                    screen_w,
                    world_x,
                );
            }
        }

        // Handle right click (context menu / cancel)
        if mouse_input.just_pressed(MouseButton::Right) {
            if in_playfield && editor_state.current_tool == EditorTool::Select {
                editor_state.deselect_all();
            }
        }
    }

    // Update UI state
    editor_ui.update_status(3);
}

/// Handle clicking on the playfield
fn handle_playfield_click(
    editor_state: &mut EditorState,
    beatmap_assets: &mut BeatmapAssets,
    world_x: f32,
    world_y: f32,
) {
    if let Some(beatmap) = beatmap_assets.current_mut() {
        match editor_state.current_tool {
            EditorTool::Select => {
                // Try to select an object
                let click_pos = Vec2::new(world_x, world_y);
                let tolerance = 25.0 * editor_state.playfield_zoom;

                if let Some(id) = editor_state.get_object_at_position(beatmap, click_pos, tolerance)
                {
                    let add_to_selection = false; // Could check for Shift key
                    editor_state.select_object(id, add_to_selection);
                } else {
                    editor_state.deselect_all();
                }
            }
            EditorTool::Circle | EditorTool::Slider | EditorTool::Spinner => {
                // Place a new object
                let mut position = Vec2::new(world_x, world_y);

                // Snap to grid if enabled
                if editor_state.snap_enabled && editor_state.show_grid {
                    position = snap_to_grid(
                        position,
                        editor_state.grid_size * editor_state.playfield_zoom,
                    );
                }

                if let Some(action) = editor_state.add_object(beatmap, position) {
                    editor_state.record_action(action);
                }
            }
            EditorTool::Delete => {
                // Delete object under cursor
                let click_pos = Vec2::new(world_x, world_y);
                let tolerance = 25.0 * editor_state.playfield_zoom;

                if let Some(id) = editor_state.get_object_at_position(beatmap, click_pos, tolerance)
                {
                    if let Some(obj) = beatmap.remove_hit_object(id) {
                        editor_state
                            .record_action(EditorAction::DeleteObjects { objects: vec![obj] });
                    }
                    editor_state.selected_objects.retain(|&x| x != id);
                }
            }
        }
    }
}

/// Handle clicking on the timeline
fn handle_timeline_click(
    editor_state: &mut EditorState,
    editor_ui: &EditorUIState,
    beatmap: Option<&crate::beatmap::Beatmap>,
    screen_w: f32,
    world_x: f32,
) {
    // Convert screen x to timeline position
    let timeline_x = world_x + screen_w / 2.0;
    let time = crate::editor::timeline_pos_to_time(
        timeline_x,
        editor_state.timeline_zoom,
        editor_state.timeline_scroll,
    );

    // Snap to beat if enabled
    let final_time = if editor_state.snap_enabled {
        if let Some(bm) = beatmap {
            bm.snap_time(time, editor_state.beat_divisor.value())
        } else {
            time
        }
    } else {
        time
    };

    editor_state.seek_to(final_time);
}

/// Handle editor interactions with UI elements
pub fn handle_editor_ui_interactions(
    mut editor_state: ResMut<EditorState>,
    mut editor_ui: ResMut<EditorUIState>,
    tool_buttons: Query<(&Transform, &ToolButton), Without<Text2d>>,
    playback_buttons: Query<(&Transform, &PlaybackButton), Without<Text2d>>,
    left_tabs: Query<(&Transform, &LeftPanelTab), Without<Text2d>>,
    right_tabs: Query<(&Transform, &RightPanelTab), Without<Text2d>>,
    timeline_objects: Query<(&Transform, &TimelineObject), Without<Text2d>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    let window = windows.single();

    if let Some(cursor_pos) = window.cursor_position() {
        let screen_w = window.width();
        let screen_h = window.height();
        let world_x = cursor_pos.x - screen_w / 2.0;
        let world_y = screen_h / 2.0 - cursor_pos.y;

        // Check for tool button clicks
        for (transform, button) in tool_buttons.iter() {
            let button_rect =
                Rect::from_center_size(transform.translation.truncate(), Vec2::new(40.0, 40.0));

            if button_rect.contains(Vec2::new(world_x, world_y))
                && mouse_input.just_pressed(MouseButton::Left)
            {
                editor_state.set_tool(button.tool);
            }
        }

        // Check for playback button clicks
        for (transform, button) in playback_buttons.iter() {
            let button_rect =
                Rect::from_center_size(transform.translation.truncate(), Vec2::new(35.0, 35.0));

            if button_rect.contains(Vec2::new(world_x, world_y))
                && mouse_input.just_pressed(MouseButton::Left)
            {
                match button {
                    PlaybackButton::PlayPause => editor_state.toggle_playback(),
                    PlaybackButton::Previous => {
                        if let Some(beatmap) = &editor_state.current_beatmap_path {
                            // Need to access beatmap, skipping for now
                        }
                    }
                    PlaybackButton::Next => {
                        // Similar
                    }
                    PlaybackButton::Stop => editor_state.stop(),
                }
            }
        }

        // Check for left panel tab clicks
        for (transform, tab) in left_tabs.iter() {
            let tab_rect =
                Rect::from_center_size(transform.translation.truncate(), Vec2::new(70.0, 28.0));

            if tab_rect.contains(Vec2::new(world_x, world_y))
                && mouse_input.just_pressed(MouseButton::Left)
            {
                editor_ui.left_panel_tab = tab.tab;
            }
        }

        // Check for right panel tab clicks
        for (transform, tab) in right_tabs.iter() {
            let tab_rect =
                Rect::from_center_size(transform.translation.truncate(), Vec2::new(80.0, 28.0));

            if tab_rect.contains(Vec2::new(world_x, world_y))
                && mouse_input.just_pressed(MouseButton::Left)
            {
                editor_ui.right_panel_tab = tab.tab;
            }
        }

        // Check for timeline object clicks
        for (transform, obj) in timeline_objects.iter() {
            let obj_rect =
                Rect::from_center_size(transform.translation.truncate(), Vec2::new(8.0, 20.0));

            if obj_rect.contains(Vec2::new(world_x, world_y))
                && mouse_input.just_pressed(MouseButton::Left)
            {
                editor_state.select_object(obj.id, false);
            }
        }
    }
}

/// Update editor (called every frame)
pub fn update_editor(
    mut editor_state: ResMut<EditorState>,
    mut editor_ui: ResMut<EditorUIState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Auto-save indicator or periodic tasks could go here

    // Check for shift key for multi-select
    let _shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
}

/// Save beatmap shortcut
pub fn handle_save_shortcut(
    editor_state: Res<EditorState>,
    mut beatmap_assets: ResMut<BeatmapAssets>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && keyboard.just_pressed(KeyCode::KeyS)
    {
        if let Some(path) = &editor_state.current_beatmap_path {
            match beatmap_assets.save(path) {
                Ok(_) => {
                    println!("Beatmap saved successfully!");
                }
                Err(e) => {
                    eprintln!("Failed to save beatmap: {}", e);
                }
            }
        }
    }
}
