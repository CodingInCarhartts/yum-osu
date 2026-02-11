use crate::analytics::{Analytics, AnalyticsState, AnalyticsView, Grade};
use crate::config::{
    get_available_keys, BackgroundStyle, GameConfig, KeyBindingType, SettingsState, SettingsTab,
};
use crate::constants::*;
use crate::structs::{
    EndData, EndState, FloatingText, GameAssets, GameStateResource, LoadingData, PracticeMenuState,
    ReadyToPlayData, SongSelectionState, VisualizingData, VisualizingState,
};
use crate::{AppState, MenuData};
use bevy::prelude::*;
use std::fs;

/// Component marker for UI elements that should be cleaned up between states
#[derive(Component)]
pub struct UiElement;

/// Component for menu buttons
#[derive(Component)]
pub struct MenuButton {
    pub action: MenuAction,
}

#[derive(Debug, Clone, Copy)]
pub enum MenuAction {
    StartGame,
    Practice,
    BeatmapEditor,
    Analytics,
    Settings,
    Exit,
}

/// Setup the main menu UI
pub fn setup_menu_ui(mut commands: Commands, assets: Res<GameAssets>, windows: Query<&Window>) {
    if let Ok(window) = windows.get_single() {
        let scr_width = window.width();
        let scr_height = window.height();

        let button_width = BUTTON_WIDTH;
        let button_height = BUTTON_HEIGHT;
        let button_spacing = BUTTON_SPACING;
        let start_y = scr_height * 0.4;

        // Title
        commands.spawn((
            Text2d::new("YumOsu!"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 72.0,
                ..default()
            },
            TextColor(NEON_PINK.into()),
            Transform::from_xyz(0.0, scr_height * 0.2, 1.0),
            UiElement,
        ));

        // Menu buttons
        let buttons = vec![
            ("Start Game", MenuAction::StartGame, start_y),
            (
                "Practice",
                MenuAction::Practice,
                start_y + button_height + button_spacing,
            ),
            (
                "Beatmap Editor",
                MenuAction::BeatmapEditor,
                start_y + 2.0 * (button_height + button_spacing),
            ),
            (
                "Analytics",
                MenuAction::Analytics,
                start_y + 3.0 * (button_height + button_spacing),
            ),
            (
                "Settings",
                MenuAction::Settings,
                start_y + 4.0 * (button_height + button_spacing),
            ),
            (
                "Exit",
                MenuAction::Exit,
                start_y + 5.0 * (button_height + button_spacing),
            ),
        ];

        for (label, action, y_pos) in buttons {
            let button_x = 0.0; // Centered

            // Button background
            commands.spawn((
                Sprite {
                    color: NEON_BLUE,
                    custom_size: Some(Vec2::new(button_width, button_height)),
                    ..default()
                },
                Transform::from_xyz(
                    button_x,
                    y_pos - scr_height / 2.0 + button_height / 2.0,
                    0.5,
                ),
                UiElement,
                MenuButton { action },
            ));

            // Button text
            commands.spawn((
                Text2d::new(label),
                TextFont {
                    font: assets.cyberpunk_font.clone(),
                    font_size: CYBERPUNK_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
                Transform::from_xyz(
                    button_x,
                    y_pos - scr_height / 2.0 + button_height / 2.0,
                    1.0,
                ),
                UiElement,
            ));
        }
    }
}

/// Handle menu interactions
pub fn handle_menu_interactions(
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameStateResource>,
    query: Query<(&Transform, &MenuButton), Without<Text2d>>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if let Ok(window) = windows.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert to world coordinates (center is 0,0 in Bevy)
            let world_x = cursor_pos.x - window.width() / 2.0;
            let world_y = window.height() / 2.0 - cursor_pos.y;

            for (transform, button) in query.iter() {
                let button_rect = Rect::from_center_size(
                    transform.translation.truncate(),
                    Vec2::new(BUTTON_WIDTH, BUTTON_HEIGHT),
                );

                if button_rect.contains(Vec2::new(world_x, world_y)) {
                    if mouse_input.just_pressed(MouseButton::Left) {
                        match button.action {
                            MenuAction::StartGame => {
                                game_state.songs = load_songs_from_assets();
                                next_state.set(AppState::SongSelection);
                            }
                            MenuAction::Practice => {
                                game_state.songs = load_songs_from_assets();
                                next_state.set(AppState::PracticeMenu);
                            }
                            MenuAction::BeatmapEditor => {
                                next_state.set(AppState::BeatmapSelection);
                            }
                            MenuAction::Analytics => {
                                next_state.set(AppState::Analytics);
                            }
                            MenuAction::Settings => {
                                next_state.set(AppState::Settings);
                            }
                            MenuAction::Exit => {
                                // Exit is handled by AppExit event
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Load all songs from the assets directory
pub fn load_songs_from_assets() -> Vec<String> {
    let mut songs = Vec::new();
    if let Ok(entries) = fs::read_dir("src/assets/music/") {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "mp3" || ext == "ogg" || ext == "wav" {
                    let full_path = entry.path().to_string_lossy().to_string();
                    songs.push(full_path.clone());
                    println!("Loaded song: {}", full_path.clone());
                }
            }
        }
    }
    songs.sort();
    songs
}

/// Cleanup UI elements
pub fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<UiElement>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Setup song selection UI
pub fn setup_song_selection_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    windows: Query<&Window>,
    game_state: Res<GameStateResource>,
) {
    if let Ok(window) = windows.get_single() {
        let screen_h = window.height();
        let screen_w = window.width();

        // Title
        commands.spawn((
            Text2d::new("Select a Song"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: CYBERPUNK_FONT_SIZE,
                ..default()
            },
            TextColor(NEON_PINK.into()),
            Transform::from_xyz(-screen_w / 2.0 + 20.0, screen_h / 2.0 - screen_h * 0.1, 1.0),
            UiElement,
        ));

        // Song list
        for (i, song) in game_state.songs.iter().enumerate() {
            let button_y =
                screen_h / 2.0 - screen_h * 0.2 - (i as f32) * (SONG_ENTRY_HEIGHT + 20.0);

            let song_name = song
                .split('/')
                .last()
                .unwrap_or(song)
                .to_uppercase()
                .replace(".MP3", "")
                .replace(".mp3", "");

            commands.spawn((
                Text2d::new(song_name),
                TextFont {
                    font: assets.cyberpunk_font.clone(),
                    font_size: CYBERPUNK_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
                Transform::from_xyz(-screen_w / 2.0 + 50.0, button_y, 1.0),
                UiElement,
                SongButton {
                    song_path: song.clone(),
                },
            ));
        }

        // Back button text
        commands.spawn((
            Text2d::new("Press ESC to go back"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5).into()),
            Transform::from_xyz(-screen_w / 2.0 + 20.0, -screen_h / 2.0 + 20.0, 1.0),
            UiElement,
        ));
    }
}

#[derive(Component)]
pub struct SongButton {
    pub song_path: String,
}

/// Handle song selection interactions
pub fn handle_song_selection(
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameStateResource>,
    query: Query<(&Transform, &SongButton), With<Text2d>>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if let Ok(window) = windows.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            let world_x = cursor_pos.x - window.width() / 2.0;
            let world_y = window.height() / 2.0 - cursor_pos.y;

            for (transform, button) in query.iter() {
                let rect = Rect::from_center_size(
                    transform.translation.truncate(),
                    Vec2::new(400.0, SONG_ENTRY_HEIGHT),
                );

                if rect.contains(Vec2::new(world_x, world_y)) {
                    if mouse_input.just_pressed(MouseButton::Left) {
                        game_state.selected_song = button.song_path.clone();
                        next_state.set(AppState::Playing);
                    }
                }
            }
        }
    }
}

/// Setup loading screen
pub fn setup_loading_ui(mut commands: Commands, assets: Res<GameAssets>, windows: Query<&Window>) {
    if let Ok(window) = windows.get_single() {
        commands.spawn((
            Text2d::new("Loading..."),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: CYBERPUNK_FONT_SIZE,
                ..default()
            },
            TextColor(NEON_PINK.into()),
            Transform::from_xyz(0.0, 0.0, 1.0),
            UiElement,
            LoadingText,
        ));
    }
}

#[derive(Component)]
pub struct LoadingText;

/// Setup ready to play countdown
pub fn setup_ready_ui(mut commands: Commands, assets: Res<GameAssets>, windows: Query<&Window>) {
    if let Ok(window) = windows.get_single() {
        commands.spawn((
            Text2d::new("Starting in 5"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: FONT_SIZE as f32,
                ..default()
            },
            TextColor(NEON_GREEN.into()),
            Transform::from_xyz(0.0, 0.0, 1.0),
            UiElement,
            CountdownText,
        ));
    }
}

#[derive(Component)]
pub struct CountdownText;

/// Update countdown
pub fn update_countdown(
    mut query: Query<&mut Text2d, With<CountdownText>>,
    ready_data: Res<ReadyToPlayData>,
) {
    let elapsed = ready_data.ready_time.elapsed().as_secs_f32();
    let remaining = (COUNTDOWN_DURATION - elapsed as f64).max(0.0) as i32;

    for mut text in query.iter_mut() {
        text.0 = format!("Starting in {}", remaining);
    }
}

/// Draw the score
pub fn draw_score_bevy(
    commands: &mut Commands,
    score: i32,
    combo: u32,
    max_combo: u32,
    assets: &GameAssets,
) {
    // Combo display
    if combo > 0 {
        let combo_text = format!("{}x", combo);
        let combo_size = if combo >= 100 {
            48.0
        } else if combo >= 50 {
            40.0
        } else if combo >= 25 {
            36.0
        } else {
            32.0
        };

        let combo_color = if combo >= 100 {
            Color::srgba(1.0, 0.84, 0.0, 1.0)
        } else if combo >= 50 {
            NEON_PINK
        } else if combo >= 25 {
            NEON_PURPLE
        } else {
            NEON_BLUE
        };

        commands.spawn((
            Text2d::new(combo_text),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: combo_size,
                ..default()
            },
            TextColor(combo_color.into()),
            Transform::from_xyz(DRAW_SCORE_X, DRAW_SCORE_Y + 50.0, 1.0),
            UiElement,
        ));
    }

    // Score display
    let score_text = format!("Score: {}", score);
    commands.spawn((
        Text2d::new(score_text),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: SCORE_FONT_SIZE,
            ..default()
        },
        TextColor(NEON_BLUE.into()),
        Transform::from_xyz(DRAW_SCORE_X, DRAW_SCORE_Y, 1.0),
        UiElement,
    ));

    // Max combo
    let max_combo_text = format!("Max Combo: {}", max_combo);
    commands.spawn((
        Text2d::new(max_combo_text),
        TextFont {
            font: assets.cyberpunk_font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6).into()),
        Transform::from_xyz(DRAW_SCORE_X, DRAW_SCORE_Y - 30.0, 1.0),
        UiElement,
    ));
}

/// Draw floating texts
pub fn draw_floating_texts_bevy(
    commands: &mut Commands,
    floating_texts: &mut Vec<FloatingText>,
    elapsed: f64,
    assets: &GameAssets,
) {
    let mut i = 0;
    while i < floating_texts.len() {
        let text = &floating_texts[i];
        let time_since_spawn = elapsed - text.spawn_time;

        if time_since_spawn >= text.duration {
            floating_texts.swap_remove(i);
            continue;
        }

        let y_offset = (time_since_spawn * 30.0) as f32;
        let alpha = 1.0 - ((time_since_spawn / text.duration) as f32);
        let color = Color::srgba(text.color.0, text.color.1, text.color.2, alpha);

        commands.spawn((
            Text2d::new(text.text.clone()),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(color.into()),
            Transform::from_xyz(text.position.x, text.position.y - y_offset, 1.0),
            UiElement,
        ));

        i += 1;
    }
}

/// Setup settings UI
pub fn setup_settings_ui(mut commands: Commands, assets: Res<GameAssets>, windows: Query<&Window>) {
    if let Ok(window) = windows.get_single() {
        let screen_h = window.height();
        let screen_w = window.width();

        commands.spawn((
            Text2d::new("Settings"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 36.0,
                ..default()
            },
            TextColor(NEON_PINK.into()),
            Transform::from_xyz(0.0, screen_h / 2.0 - 60.0, 1.0),
            UiElement,
        ));

        commands.spawn((
            Text2d::new("Press ESC to go back"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5).into()),
            Transform::from_xyz(-screen_w / 2.0 + 20.0, -screen_h / 2.0 + 20.0, 1.0),
            UiElement,
        ));
    }
}

/// Setup practice menu UI
pub fn setup_practice_menu_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.get_single() {
        let screen_h = window.height();
        let screen_w = window.width();

        commands.spawn((
            Text2d::new("Practice Mode"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 36.0,
                ..default()
            },
            TextColor(NEON_YELLOW.into()),
            Transform::from_xyz(0.0, screen_h / 2.0 - 60.0, 1.0),
            UiElement,
        ));

        commands.spawn((
            Text2d::new("Press ESC to go back"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5).into()),
            Transform::from_xyz(-screen_w / 2.0 + 20.0, -screen_h / 2.0 + 20.0, 1.0),
            UiElement,
        ));
    }
}

/// Setup analytics UI
pub fn setup_analytics_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.get_single() {
        let screen_h = window.height();
        let screen_w = window.width();

        commands.spawn((
            Text2d::new("Analytics"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 36.0,
                ..default()
            },
            TextColor(NEON_PINK.into()),
            Transform::from_xyz(0.0, screen_h / 2.0 - 60.0, 1.0),
            UiElement,
        ));

        commands.spawn((
            Text2d::new("Press ESC to go back"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5).into()),
            Transform::from_xyz(-screen_w / 2.0 + 20.0, -screen_h / 2.0 + 20.0, 1.0),
            UiElement,
        ));
    }
}

/// Setup end screen UI
pub fn setup_end_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    windows: Query<&Window>,
    end_data: Res<EndData>,
) {
    if let Ok(window) = windows.get_single() {
        let scr_width = window.width();
        let scr_height = window.height();

        // Title
        commands.spawn((
            Text2d::new("Results"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 48.0,
                ..default()
            },
            TextColor(NEON_PINK.into()),
            Transform::from_xyz(0.0, scr_height * 0.3, 1.0),
            UiElement,
        ));

        // Score
        commands.spawn((
            Text2d::new(format!("Score: {}", end_data.state.score)),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 32.0,
                ..default()
            },
            TextColor(NEON_BLUE.into()),
            Transform::from_xyz(0.0, scr_height * 0.1, 1.0),
            UiElement,
        ));

        // Grade
        commands.spawn((
            Text2d::new(format!("Grade: {}", end_data.state.grade.as_str())),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 40.0,
                ..default()
            },
            TextColor(get_grade_color(end_data.state.grade.as_str()).into()),
            Transform::from_xyz(0.0, 0.0, 1.0),
            UiElement,
        ));

        // Accuracy
        commands.spawn((
            Text2d::new(format!("Accuracy: {:.1}%", end_data.state.accuracy)),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(NEON_GREEN.into()),
            Transform::from_xyz(0.0, -scr_height * 0.1, 1.0),
            UiElement,
        ));

        // Continue prompt
        commands.spawn((
            Text2d::new("Click or press ENTER to continue"),
            TextFont {
                font: assets.cyberpunk_font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7).into()),
            Transform::from_xyz(0.0, -scr_height * 0.3, 1.0),
            UiElement,
        ));
    }
}
