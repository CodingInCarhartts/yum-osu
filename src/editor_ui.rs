// src/editor_ui.rs

use crate::beatmap::{BeatDivisor, Beatmap, EditorTool, HitObjectKind};
use crate::constants::*;
use crate::editor::{
    grid_to_screen, snap_to_grid, EditorAction, EditorLeftTab, EditorRightTab, EditorState,
    EditorUIState,
};
use crate::structs::GameAssets;
use crate::ui::UiElement;
use bevy::prelude::*;
use bevy::window::Window;

/// Setup the editor UI
pub fn setup_editor_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    windows: Query<&Window>,
    editor_ui: Res<EditorUIState>,
    editor_state: Res<EditorState>,
    beatmap_assets: Res<crate::beatmap::BeatmapAssets>,
) {
    let window = windows.single();
    let screen_w = window.width();
    let screen_h = window.height();

    // Background
    commands.spawn((
        Sprite {
            color: DARK_BACKGROUND,
            custom_size: Some(Vec2::new(screen_w, screen_h)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        UiElement,
    ));

    // Toolbar
    spawn_toolbar(
        &mut commands,
        &assets,
        screen_w,
        screen_h,
        &editor_state,
        &editor_ui,
    );

    // Left panel (tools/timing/bookmarks)
    if editor_ui.left_panel_visible {
        spawn_left_panel(&mut commands, &assets, &editor_ui, &editor_state, screen_h);
    }

    // Right panel (properties)
    if editor_ui.right_panel_visible {
        spawn_right_panel(
            &mut commands,
            &assets,
            &editor_ui,
            &editor_state,
            beatmap_assets.current(),
            screen_w,
            screen_h,
        );
    }

    // Timeline at bottom
    spawn_timeline(
        &mut commands,
        &assets,
        &editor_state,
        &editor_ui,
        screen_w,
        screen_h,
        beatmap_assets.current(),
    );

    // Playfield grid
    spawn_playfield_grid(&mut commands, &assets, &editor_state, screen_w, screen_h);

    // Status bar
    spawn_status_bar(
        &mut commands,
        &assets,
        &editor_state,
        beatmap_assets.current(),
        screen_w,
        screen_h,
    );
}

/// Spawn the toolbar
fn spawn_toolbar(
    commands: &mut Commands,
    assets: &GameAssets,
    screen_w: f32,
    screen_h: f32,
    editor_state: &EditorState,
    editor_ui: &EditorUIState,
) {
    let toolbar_y = screen_h / 2.0 - editor_ui.toolbar_height / 2.0;

    // Toolbar background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.1, 0.15, 1.0),
            custom_size: Some(Vec2::new(screen_w, editor_ui.toolbar_height)),
            ..default()
        },
        Transform::from_xyz(0.0, toolbar_y, 0.1),
        UiElement,
        EditorToolbar,
    ));

    // Tool buttons
    let tools = EditorTool::all();
    let button_size = 40.0;
    let button_spacing = 10.0;
    let start_x = -screen_w / 2.0 + 20.0 + button_size / 2.0;

    for (i, tool) in tools.iter().enumerate() {
        let x = start_x + i as f32 * (button_size + button_spacing);
        let is_active = editor_state.current_tool == *tool;
        let color = if is_active { NEON_PINK } else { NEON_BLUE };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(button_size, button_size)),
                ..default()
            },
            Transform::from_xyz(x, toolbar_y, 0.2),
            UiElement,
            ToolButton { tool: *tool },
        ));

        commands.spawn((
            Text2d::new(tool.display_name()),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            Transform::from_xyz(x, toolbar_y - button_size / 2.0 - 10.0, 0.3),
            UiElement,
        ));
    }

    // Playback controls
    let play_x = screen_w / 2.0 - 150.0;
    spawn_playback_controls(commands, assets, play_x, toolbar_y, editor_state);

    // Beat divisor selector
    let divisor_x = 0.0;
    spawn_divisor_selector(commands, assets, divisor_x, toolbar_y, editor_state);
}

/// Spawn playback controls
fn spawn_playback_controls(
    commands: &mut Commands,
    assets: &GameAssets,
    center_x: f32,
    y: f32,
    editor_state: &EditorState,
) {
    let button_size = 35.0;
    let spacing = 5.0;

    // Previous beat button
    commands.spawn((
        Sprite {
            color: NEON_BLUE,
            custom_size: Some(Vec2::new(button_size, button_size)),
            ..default()
        },
        Transform::from_xyz(center_x - button_size - spacing, y, 0.2),
        UiElement,
        PlaybackButton::Previous,
    ));

    // Play/Pause button
    let play_color = if editor_state.is_playing {
        NEON_GREEN
    } else {
        NEON_PINK
    };
    let play_text = if editor_state.is_playing { "||" } else { "▶" };
    commands.spawn((
        Sprite {
            color: play_color,
            custom_size: Some(Vec2::new(button_size * 1.2, button_size)),
            ..default()
        },
        Transform::from_xyz(center_x, y, 0.2),
        UiElement,
        PlaybackButton::PlayPause,
    ));

    commands.spawn((
        Text2d::new(play_text),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE.into()),
        Transform::from_xyz(center_x, y, 0.3),
        UiElement,
    ));

    // Next beat button
    commands.spawn((
        Sprite {
            color: NEON_BLUE,
            custom_size: Some(Vec2::new(button_size, button_size)),
            ..default()
        },
        Transform::from_xyz(center_x + button_size + spacing, y, 0.2),
        UiElement,
        PlaybackButton::Next,
    ));

    // Stop button
    commands.spawn((
        Sprite {
            color: NEON_ORANGE,
            custom_size: Some(Vec2::new(button_size, button_size)),
            ..default()
        },
        Transform::from_xyz(center_x + (button_size + spacing) * 2.0, y, 0.2),
        UiElement,
        PlaybackButton::Stop,
    ));
}

/// Spawn beat divisor selector
fn spawn_divisor_selector(
    commands: &mut Commands,
    assets: &GameAssets,
    x: f32,
    y: f32,
    editor_state: &EditorState,
) {
    commands.spawn((
        Text2d::new(format!(
            "Beat Snap: {}",
            editor_state.beat_divisor.display_name()
        )),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(NEON_CYAN.into()),
        Transform::from_xyz(x, y, 0.2),
        UiElement,
        BeatDivisorDisplay,
    ));

    // Snap toggle
    let snap_text = if editor_state.snap_enabled {
        "[Snap: ON]"
    } else {
        "[Snap: OFF]"
    };
    let snap_color = if editor_state.snap_enabled {
        NEON_GREEN
    } else {
        Color::GRAY
    };
    commands.spawn((
        Text2d::new(snap_text),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(snap_color.into()),
        Transform::from_xyz(x, y - 15.0, 0.2),
        UiElement,
        SnapToggleButton,
    ));
}

/// Spawn left panel
fn spawn_left_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    editor_ui: &EditorUIState,
    editor_state: &EditorState,
    screen_h: f32,
) {
    let panel_x = -screen_h / 2.0 + editor_ui.left_panel_width / 2.0;
    let panel_y = 0.0;
    let content_height = screen_h - editor_ui.toolbar_height - editor_ui.timeline_height - 40.0;

    // Panel background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.08, 0.08, 0.12, 1.0),
            custom_size: Some(Vec2::new(editor_ui.left_panel_width, content_height)),
            ..default()
        },
        Transform::from_xyz(panel_x, panel_y, 0.1),
        UiElement,
        LeftPanel,
    ));

    // Tab buttons
    let tabs = vec![
        (EditorLeftTab::Tools, "Tools"),
        (EditorLeftTab::Timing, "Timing"),
        (EditorLeftTab::Bookmarks, "Bookmarks"),
    ];

    let tab_width = editor_ui.left_panel_width / tabs.len() as f32;
    for (i, (tab, name)) in tabs.iter().enumerate() {
        let is_active = editor_ui.left_panel_tab == *tab;
        let color = if is_active {
            NEON_PINK
        } else {
            Color::srgba(0.15, 0.15, 0.2, 1.0)
        };
        let tab_x = panel_x - editor_ui.left_panel_width / 2.0 + tab_width * (i as f32 + 0.5);
        let tab_y = content_height / 2.0 - 15.0;

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(tab_width - 2.0, 28.0)),
                ..default()
            },
            Transform::from_xyz(tab_x, tab_y, 0.2),
            UiElement,
            LeftPanelTab { tab: *tab },
        ));

        commands.spawn((
            Text2d::new(*name),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            Transform::from_xyz(tab_x, tab_y, 0.3),
            UiElement,
        ));
    }

    // Panel content based on selected tab
    match editor_ui.left_panel_tab {
        EditorLeftTab::Tools => {
            spawn_tools_panel(commands, assets, panel_x, panel_y, editor_ui, editor_state)
        }
        EditorLeftTab::Timing => {
            spawn_timing_panel(commands, assets, panel_x, panel_y, editor_ui, editor_state)
        }
        EditorLeftTab::Bookmarks => {
            spawn_bookmarks_panel(commands, assets, panel_x, panel_y, editor_ui)
        }
    }
}

/// Spawn tools panel content
fn spawn_tools_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    panel_x: f32,
    panel_y: f32,
    editor_ui: &EditorUIState,
    editor_state: &EditorState,
) {
    let start_y = panel_y + editor_ui.left_panel_width / 2.0 - 50.0;

    // New Combo toggle
    let combo_color = if editor_state.new_combo_mode {
        NEON_GREEN
    } else {
        Color::GRAY
    };
    commands.spawn((
        Text2d::new("New Combo (Q)"),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(combo_color.into()),
        Transform::from_xyz(panel_x, start_y, 0.2),
        UiElement,
        NewComboToggle,
    ));

    // Hitsound selector
    commands.spawn((
        Text2d::new(format!("Hitsound: {:?}", editor_state.current_hitsound)),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(NEON_CYAN.into()),
        Transform::from_xyz(panel_x, start_y - 30.0, 0.2),
        UiElement,
    ));

    // Grid settings
    commands.spawn((
        Text2d::new(format!("Grid Size: {:.0}px", editor_state.grid_size)),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::WHITE.into()),
        Transform::from_xyz(panel_x, start_y - 60.0, 0.2),
        UiElement,
    ));

    let grid_toggle_color = if editor_state.show_grid {
        NEON_GREEN
    } else {
        Color::GRAY
    };
    commands.spawn((
        Text2d::new("Show Grid (G)"),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(grid_toggle_color.into()),
        Transform::from_xyz(panel_x, start_y - 85.0, 0.2),
        UiElement,
        GridToggle,
    ));
}

/// Spawn timing panel content
fn spawn_timing_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    panel_x: f32,
    panel_y: f32,
    editor_ui: &EditorUIState,
    editor_state: &EditorState,
) {
    commands.spawn((
        Text2d::new("Timing Points"),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(NEON_PINK.into()),
        Transform::from_xyz(panel_x, panel_y + 80.0, 0.2),
        UiElement,
    ));
}

/// Spawn bookmarks panel content
fn spawn_bookmarks_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    panel_x: f32,
    panel_y: f32,
    editor_ui: &EditorUIState,
) {
    commands.spawn((
        Text2d::new("Bookmarks"),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(NEON_PINK.into()),
        Transform::from_xyz(panel_x, panel_y + 80.0, 0.2),
        UiElement,
    ));
}

/// Spawn right panel
fn spawn_right_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    editor_ui: &EditorUIState,
    editor_state: &EditorState,
    beatmap: Option<&Beatmap>,
    screen_w: f32,
    screen_h: f32,
) {
    let panel_x = screen_w / 2.0 - editor_ui.right_panel_width / 2.0;
    let panel_y = 0.0;
    let content_height = screen_h - editor_ui.toolbar_height - editor_ui.timeline_height - 40.0;

    // Panel background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.08, 0.08, 0.12, 1.0),
            custom_size: Some(Vec2::new(editor_ui.right_panel_width, content_height)),
            ..default()
        },
        Transform::from_xyz(panel_x, panel_y, 0.1),
        UiElement,
        RightPanel,
    ));

    // Tab buttons
    let tabs = vec![
        (EditorRightTab::Properties, "Properties"),
        (EditorRightTab::Settings, "Settings"),
        (EditorRightTab::Metadata, "Metadata"),
    ];

    let tab_width = editor_ui.right_panel_width / tabs.len() as f32;
    for (i, (tab, name)) in tabs.iter().enumerate() {
        let is_active = editor_ui.right_panel_tab == *tab;
        let color = if is_active {
            NEON_PINK
        } else {
            Color::srgba(0.15, 0.15, 0.2, 1.0)
        };
        let tab_x = panel_x - editor_ui.right_panel_width / 2.0 + tab_width * (i as f32 + 0.5);
        let tab_y = content_height / 2.0 - 15.0;

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(tab_width - 2.0, 28.0)),
                ..default()
            },
            Transform::from_xyz(tab_x, tab_y, 0.2),
            UiElement,
            RightPanelTab { tab: *tab },
        ));

        commands.spawn((
            Text2d::new(*name),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            Transform::from_xyz(tab_x, tab_y, 0.3),
            UiElement,
        ));
    }

    // Panel content
    if let Some(beatmap) = beatmap {
        match editor_ui.right_panel_tab {
            EditorRightTab::Properties => spawn_properties_panel(
                commands,
                assets,
                panel_x,
                panel_y,
                beatmap,
                editor_state,
                editor_ui,
            ),
            EditorRightTab::Settings => {
                spawn_settings_panel(commands, assets, panel_x, panel_y, beatmap, editor_ui)
            }
            EditorRightTab::Metadata => {
                spawn_metadata_panel(commands, assets, panel_x, panel_y, beatmap, editor_ui)
            }
        }
    }
}

/// Spawn properties panel
fn spawn_properties_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    panel_x: f32,
    panel_y: f32,
    beatmap: &Beatmap,
    editor_state: &EditorState,
    editor_ui: &EditorUIState,
) {
    let start_y = panel_y + editor_ui.right_panel_width / 2.0 - 50.0;

    // Object count
    let stats = beatmap.get_object_stats();
    commands.spawn((
        Text2d::new(format!("Objects: {}", stats.total)),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::WHITE.into()),
        Transform::from_xyz(panel_x, start_y, 0.2),
        UiElement,
    ));

    commands.spawn((
        Text2d::new(format!(
            "Circles: {} | Sliders: {} | Spinners: {}",
            stats.circles, stats.sliders, stats.spinners
        )),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0).into()),
        Transform::from_xyz(panel_x, start_y - 15.0, 0.2),
        UiElement,
    ));

    // Duration
    let duration = beatmap.get_duration();
    let minutes = (duration / 60.0) as u32;
    let seconds = (duration % 60.0) as u32;
    commands.spawn((
        Text2d::new(format!("Duration: {:02}:{:02}", minutes, seconds)),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::WHITE.into()),
        Transform::from_xyz(panel_x, start_y - 40.0, 0.2),
        UiElement,
    ));

    // Selected objects info
    if !editor_state.selected_objects.is_empty() {
        commands.spawn((
            Text2d::new(format!("Selected: {}", editor_state.selected_objects.len())),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(NEON_GREEN.into()),
            Transform::from_xyz(panel_x, start_y - 70.0, 0.2),
            UiElement,
        ));
    }
}

/// Spawn settings panel
fn spawn_settings_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    panel_x: f32,
    panel_y: f32,
    beatmap: &Beatmap,
    editor_ui: &EditorUIState,
) {
    let start_y = panel_y + editor_ui.right_panel_width / 2.0 - 50.0;
    let settings = &beatmap.settings;

    let settings_text = vec![
        format!("Circle Size (CS): {:.1}", settings.circle_size),
        format!("Approach Rate (AR): {:.1}", settings.approach_rate),
        format!(
            "Overall Difficulty (OD): {:.1}",
            settings.overall_difficulty
        ),
        format!("HP Drain: {:.1}", settings.hp_drain),
        format!("Slider Multiplier: {:.2}x", settings.slider_multiplier),
    ];

    for (i, text) in settings_text.iter().enumerate() {
        commands.spawn((
            Text2d::new(text.clone()),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            Transform::from_xyz(panel_x, start_y - i as f32 * 20.0, 0.2),
            UiElement,
        ));
    }
}

/// Spawn metadata panel
fn spawn_metadata_panel(
    commands: &mut Commands,
    assets: &GameAssets,
    panel_x: f32,
    panel_y: f32,
    beatmap: &Beatmap,
    editor_ui: &EditorUIState,
) {
    let start_y = panel_y + editor_ui.right_panel_width / 2.0 - 50.0;
    let meta = &beatmap.metadata;

    let meta_text = vec![
        format!("Title: {}", meta.title),
        format!("Artist: {}", meta.artist),
        format!("Creator: {}", meta.creator),
        format!("Version: {}", meta.version),
    ];

    for (i, text) in meta_text.iter().enumerate() {
        commands.spawn((
            Text2d::new(text.clone()),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::WHITE.into()),
            Transform::from_xyz(panel_x, start_y - i as f32 * 20.0, 0.2),
            UiElement,
        ));
    }
}

/// Spawn timeline
fn spawn_timeline(
    commands: &mut Commands,
    assets: &GameAssets,
    editor_state: &EditorState,
    editor_ui: &EditorUIState,
    screen_w: f32,
    screen_h: f32,
    beatmap: Option<&Beatmap>,
) {
    let timeline_y = -screen_h / 2.0 + editor_ui.timeline_height / 2.0 + 20.0;

    // Timeline background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.08, 1.0),
            custom_size: Some(Vec2::new(screen_w, editor_ui.timeline_height)),
            ..default()
        },
        Transform::from_xyz(0.0, timeline_y, 0.1),
        UiElement,
        Timeline,
    ));

    // Time markers
    if let Some(beatmap) = beatmap {
        let zoom = editor_state.timeline_zoom;
        let scroll = editor_state.timeline_scroll;
        let visible_start = crate::editor::timeline_pos_to_time(0.0, zoom, scroll);
        let visible_end = crate::editor::timeline_pos_to_time(screen_w, zoom, scroll);

        // Draw beat lines
        let beat_length = beatmap.get_beat_length_at(visible_start);
        let start_beat = (visible_start / beat_length).floor() as i32;
        let end_beat = (visible_end / beat_length).ceil() as i32;

        for beat in start_beat..=end_beat {
            let time = beat as f64 * beat_length;
            let x = crate::editor::time_to_timeline_pos(time, zoom, scroll) - screen_w / 2.0;

            if x > -screen_w / 2.0 && x < screen_w / 2.0 {
                let opacity = crate::editor::get_beat_line_opacity(beat as usize);
                let height = if beat % 4 == 0 {
                    editor_ui.timeline_height * 0.8
                } else {
                    editor_ui.timeline_height * 0.4
                };

                commands.spawn((
                    Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, opacity * 0.3),
                        custom_size: Some(Vec2::new(1.0, height)),
                        ..default()
                    },
                    Transform::from_xyz(x, timeline_y, 0.15),
                    UiElement,
                ));
            }
        }

        // Draw hit objects on timeline
        for obj in &beatmap.hit_objects {
            if obj.time >= visible_start && obj.time <= visible_end {
                let x =
                    crate::editor::time_to_timeline_pos(obj.time, zoom, scroll) - screen_w / 2.0;

                let color = match obj.kind {
                    HitObjectKind::Circle => NEON_BLUE,
                    HitObjectKind::Slider { .. } => NEON_PURPLE,
                    HitObjectKind::Spinner { .. } => NEON_YELLOW,
                };

                let is_selected = editor_state.selected_objects.contains(&obj.id);
                let height = if is_selected { 20.0 } else { 14.0 };
                let z = if is_selected { 0.25 } else { 0.2 };

                commands.spawn((
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(4.0, height)),
                        ..default()
                    },
                    Transform::from_xyz(x, timeline_y, z),
                    UiElement,
                    TimelineObject { id: obj.id },
                ));
            }
        }
    }

    // Playhead
    let playhead_x = 0.0; // Center of screen
    commands.spawn((
        Sprite {
            color: NEON_PINK,
            custom_size: Some(Vec2::new(2.0, editor_ui.timeline_height)),
            ..default()
        },
        Transform::from_xyz(playhead_x, timeline_y, 0.3),
        UiElement,
        Playhead,
    ));

    // Current time display
    let minutes = (editor_state.current_time / 60.0) as u32;
    let seconds = (editor_state.current_time % 60.0) as u32;
    let millis = ((editor_state.current_time % 1.0) * 1000.0) as u32;
    let time_str = format!("{:02}:{:02}.{:03}", minutes, seconds, millis);

    commands.spawn((
        Text2d::new(time_str),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(NEON_PINK.into()),
        Transform::from_xyz(
            0.0,
            timeline_y + editor_ui.timeline_height / 2.0 + 10.0,
            0.3,
        ),
        UiElement,
        TimeDisplay,
    ));
}

/// Spawn playfield grid
fn spawn_playfield_grid(
    commands: &mut Commands,
    assets: &GameAssets,
    editor_state: &EditorState,
    screen_w: f32,
    screen_h: f32,
) {
    if !editor_state.show_grid {
        return;
    }

    let grid_cols = 16;
    let grid_rows = 12;
    let grid_size = editor_state.grid_size * editor_state.playfield_zoom;

    let playfield_w = grid_cols as f32 * grid_size;
    let playfield_h = grid_rows as f32 * grid_size;

    // Grid background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.02, 0.02, 0.04, 0.8),
            custom_size: Some(Vec2::new(playfield_w, playfield_h)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.05),
        UiElement,
        PlayfieldGrid,
    ));

    // Grid lines
    for col in 0..=grid_cols {
        let x = (col as f32 - grid_cols as f32 / 2.0) * grid_size;
        let alpha = if col % 4 == 0 { 0.3 } else { 0.1 };

        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, alpha),
                custom_size: Some(Vec2::new(1.0, playfield_h)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, 0.06),
            UiElement,
        ));
    }

    for row in 0..=grid_rows {
        let y = (row as f32 - grid_rows as f32 / 2.0) * grid_size;
        let alpha = if row % 4 == 0 { 0.3 } else { 0.1 };

        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, alpha),
                custom_size: Some(Vec2::new(playfield_w, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y, 0.06),
            UiElement,
        ));
    }
}

/// Spawn status bar
fn spawn_status_bar(
    commands: &mut Commands,
    assets: &GameAssets,
    editor_state: &EditorState,
    beatmap: Option<&Beatmap>,
    screen_w: f32,
    screen_h: f32,
) {
    let bar_y = -screen_h / 2.0 + 10.0;
    let bar_height = 20.0;

    // Status bar background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.08, 0.08, 0.12, 1.0),
            custom_size: Some(Vec2::new(screen_w, bar_height)),
            ..default()
        },
        Transform::from_xyz(0.0, bar_y, 0.1),
        UiElement,
        StatusBar,
    ));

    // Status message
    let status_text = if let Some((msg, _)) = &editor_state.status_message {
        msg.clone()
    } else if let Some(beatmap) = beatmap {
        format!(
            "{} - {} [{}] | {} objects",
            beatmap.metadata.artist,
            beatmap.metadata.title,
            beatmap.metadata.version,
            beatmap.hit_objects.len()
        )
    } else {
        "No beatmap loaded".to_string()
    };

    commands.spawn((
        Text2d::new(status_text),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0).into()),
        Transform::from_xyz(-screen_w / 2.0 + 10.0, bar_y, 0.2),
        UiElement,
        StatusText,
    ));

    // Help hint
    commands.spawn((
        Text2d::new("Press F1 for Help | ESC to Exit"),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0).into()),
        Transform::from_xyz(screen_w / 2.0 - 100.0, bar_y, 0.2),
        UiElement,
    ));
}

/// Render hit objects in the playfield
pub fn render_editor_hit_objects(
    mut commands: Commands,
    assets: Res<GameAssets>,
    editor_state: Res<EditorState>,
    beatmap_assets: Res<crate::beatmap::BeatmapAssets>,
) {
    if let Some(beatmap) = beatmap_assets.current() {
        let approach_time = beatmap.settings.get_approach_time();
        let current_time = editor_state.current_time;

        for obj in &beatmap.hit_objects {
            // Check if object is visible (within approach window)
            let time_diff = obj.time - current_time;
            if time_diff < -0.2 || time_diff > approach_time {
                continue;
            }

            let is_selected = editor_state.selected_objects.contains(&obj.id);
            let alpha = if time_diff < 0.0 {
                1.0 - ((-time_diff) / 0.2) as f32
            } else {
                1.0
            };

            let color = match obj.kind {
                HitObjectKind::Circle => {
                    if is_selected {
                        NEON_GREEN
                    } else if obj.new_combo {
                        NEON_PINK
                    } else {
                        NEON_BLUE
                    }
                }
                HitObjectKind::Slider { .. } => {
                    if is_selected {
                        NEON_GREEN
                    } else {
                        NEON_PURPLE
                    }
                }
                HitObjectKind::Spinner { .. } => {
                    if is_selected {
                        NEON_GREEN
                    } else {
                        NEON_YELLOW
                    }
                }
            };

            let radius = 20.0 * editor_state.playfield_zoom;

            // Draw approach circle
            if time_diff > 0.0 {
                let approach_scale = (time_diff / approach_time) as f32;
                let approach_radius = radius * (1.0 + approach_scale * 2.0);

                commands.spawn((
                    Sprite {
                        color: Color::srgba(
                            color.to_linear().red,
                            color.to_linear().green,
                            color.to_linear().blue,
                            approach_scale * 0.3,
                        ),
                        custom_size: Some(Vec2::new(approach_radius * 2.0, approach_radius * 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(obj.position.x, obj.position.y, 0.1),
                    UiElement,
                ));
            }

            // Draw object
            commands.spawn((
                Sprite {
                    color: Color::srgba(
                        color.to_linear().red,
                        color.to_linear().green,
                        color.to_linear().blue,
                        alpha,
                    ),
                    custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                    ..default()
                },
                Transform::from_xyz(obj.position.x, obj.position.y, 0.2),
                UiElement,
                EditorHitObject { id: obj.id },
            ));

            // Draw selection indicator
            if is_selected {
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.0, 1.0, 0.5, 0.5),
                        custom_size: Some(Vec2::new(radius * 2.5, radius * 2.5)),
                        ..default()
                    },
                    Transform::from_xyz(obj.position.x, obj.position.y, 0.15),
                    UiElement,
                ));
            }

            // Draw combo number
            if obj.combo_index > 0 {
                commands.spawn((
                    Text2d::new(obj.combo_index.to_string()),
                    TextFont {
                        font: assets.cyberpunk_font.clone(),
                        font_size: 12.0 * editor_state.playfield_zoom,
                        ..default()
                    },
                    TextColor(Color::WHITE.into()),
                    Transform::from_xyz(obj.position.x, obj.position.y, 0.3),
                    UiElement,
                ));
            }
        }
    }
}

// Component markers
#[derive(Component)]
pub struct EditorToolbar;

#[derive(Component)]
pub struct ToolButton {
    pub tool: EditorTool,
}

#[derive(Component)]
pub enum PlaybackButton {
    PlayPause,
    Previous,
    Next,
    Stop,
}

#[derive(Component)]
pub struct BeatDivisorDisplay;

#[derive(Component)]
pub struct SnapToggleButton;

#[derive(Component)]
pub struct LeftPanel;

#[derive(Component)]
pub struct LeftPanelTab {
    pub tab: EditorLeftTab,
}

#[derive(Component)]
pub struct NewComboToggle;

#[derive(Component)]
pub struct GridToggle;

#[derive(Component)]
pub struct RightPanel;

#[derive(Component)]
pub struct RightPanelTab {
    pub tab: EditorRightTab,
}

#[derive(Component)]
pub struct Timeline;

#[derive(Component)]
pub struct TimelineObject {
    pub id: HitObjectId,
}

#[derive(Component)]
pub struct Playhead;

#[derive(Component)]
pub struct TimeDisplay;

#[derive(Component)]
pub struct PlayfieldGrid;

#[derive(Component)]
pub struct StatusBar;

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct EditorHitObject {
    pub id: HitObjectId,
}

// Type alias for HitObjectId
use crate::beatmap::HitObjectId;
