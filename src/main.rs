mod analytics;
mod audio;
mod config;
mod constants;
mod game;
mod structs;
mod ui;

use crate::analytics::{Analytics, AnalyticsState};
use crate::audio::gather_beats;
use crate::config::{GameConfig, SettingsState};
use crate::constants::*;
use crate::game::*;
use crate::structs::*;
use crate::ui::*;

use bevy::prelude::*;
use bevy::window::WindowCloseRequested;
use rodio::{Decoder, OutputStream, Sink};
use std::sync::mpsc;
use std::time::Instant;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_config()))
        .init_state::<AppState>()
        .init_resource::<GameStateResource>()
        .init_resource::<GameTime>()
        .init_resource::<SettingsState>()
        .init_resource::<AnalyticsState>()
        .init_resource::<PracticeMenuState>()
        .add_event::<GameEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_window_close, update_game_time))
        // Menu state systems
        .add_systems(OnEnter(AppState::Menu), (enter_menu, setup_menu_ui))
        .add_systems(
            Update,
            (update_menu, handle_menu_interactions).run_if(in_state(AppState::Menu)),
        )
        .add_systems(OnExit(AppState::Menu), (exit_menu, cleanup_ui))
        // Song selection state systems
        .add_systems(
            OnEnter(AppState::SongSelection),
            (enter_song_selection, setup_song_selection_ui),
        )
        .add_systems(
            Update,
            (update_song_selection, handle_song_selection)
                .run_if(in_state(AppState::SongSelection)),
        )
        .add_systems(OnExit(AppState::SongSelection), cleanup_ui)
        // Practice menu state systems
        .add_systems(
            OnEnter(AppState::PracticeMenu),
            (enter_practice_menu, setup_practice_menu_ui),
        )
        .add_systems(
            Update,
            update_practice_menu.run_if(in_state(AppState::PracticeMenu)),
        )
        .add_systems(OnExit(AppState::PracticeMenu), cleanup_ui)
        // Loading state systems
        .add_systems(
            OnEnter(AppState::Loading),
            (enter_loading, setup_loading_ui),
        )
        .add_systems(Update, update_loading.run_if(in_state(AppState::Loading)))
        .add_systems(OnExit(AppState::Loading), cleanup_ui)
        // ReadyToPlay state systems
        .add_systems(
            OnEnter(AppState::ReadyToPlay),
            (enter_ready_to_play, setup_ready_ui),
        )
        .add_systems(
            Update,
            (update_ready_to_play, update_countdown).run_if(in_state(AppState::ReadyToPlay)),
        )
        .add_systems(OnExit(AppState::ReadyToPlay), cleanup_ui)
        // Visualizing state systems
        .add_systems(OnEnter(AppState::Visualizing), enter_visualizing)
        .add_systems(
            Update,
            (
                update_visualizing,
                render_game_circles,
                render_game_floating_texts,
                render_game_score,
            )
                .run_if(in_state(AppState::Visualizing)),
        )
        .add_systems(OnExit(AppState::Visualizing), exit_visualizing)
        // End state systems
        .add_systems(OnEnter(AppState::End), (enter_end, setup_end_ui))
        .add_systems(Update, update_end.run_if(in_state(AppState::End)))
        .add_systems(OnExit(AppState::End), cleanup_ui)
        // Settings state systems
        .add_systems(
            OnEnter(AppState::Settings),
            (enter_settings, setup_settings_ui),
        )
        .add_systems(Update, update_settings.run_if(in_state(AppState::Settings)))
        .add_systems(OnExit(AppState::Settings), cleanup_ui)
        // Analytics state systems
        .add_systems(
            OnEnter(AppState::Analytics),
            (enter_analytics, setup_analytics_ui),
        )
        .add_systems(
            Update,
            update_analytics.run_if(in_state(AppState::Analytics)),
        )
        .add_systems(OnExit(AppState::Analytics), cleanup_ui)
        .run();
}

/// Application states
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    SongSelection,
    PracticeMenu,
    Playing,
    Loading,
    ReadyToPlay,
    Visualizing,
    End,
    Settings,
    Analytics,
}

/// Game events for communication between systems
#[derive(Event)]
pub enum GameEvent {
    StartGame,
    SelectSong(String),
    StartPractice,
    OpenSettings,
    OpenAnalytics,
    Exit,
    BackToMenu,
}

/// Setup system - runs once at startup
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load font
    let font_handle: Handle<Font> = asset_server.load("fonts/teknaf.otf");

    // Insert resources
    commands.insert_resource(GameAssets {
        cyberpunk_font: font_handle,
    });

    // Load configuration
    let config = GameConfig::load();
    commands.insert_resource(config.clone());

    // Load analytics
    let analytics = Analytics::load();
    commands.insert_resource(analytics);

    // Setup audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    commands.insert_resource(GameAudioSink { sink });
    // Note: _stream must be kept alive, we'll store it in a resource
    commands.insert_resource(AudioStream(_stream));

    // Setup camera
    commands.spawn(Camera2d);
}

/// Resource to hold audio stream (must be kept alive)
#[derive(Resource)]
pub struct AudioStream(#[allow(dead_code)] OutputStream);

/// Update game time
fn update_game_time(mut game_time: ResMut<GameTime>) {
    game_time.elapsed = game_time.start_time.elapsed().as_secs_f64();
}

/// Handle window close
fn handle_window_close(
    mut events: EventReader<WindowCloseRequested>,
    config: Res<GameConfig>,
    analytics: Res<Analytics>,
    mut app_exit: EventWriter<AppExit>,
) {
    for _ in events.read() {
        // Save config and analytics before exit
        config.save();
        analytics.save();
        app_exit.send(AppExit::Success);
    }
}

// ==================== MENU STATE ====================

fn enter_menu(mut commands: Commands) {
    commands.insert_resource(MenuData::default());
}

#[derive(Resource, Default)]
pub struct MenuData {
    pub buttons: Vec<(String, Rect)>,
}

fn update_menu(windows: Query<&Window>, mut menu_data: ResMut<MenuData>) {
    if let Ok(window) = windows.single() {
        let scr_width = window.width();
        let scr_height = window.height();

        let button_width = BUTTON_WIDTH;
        let button_height = BUTTON_HEIGHT;
        let button_spacing = BUTTON_SPACING;
        let start_y = scr_height * 0.4;

        let buttons = vec![
            ("Start Game".to_string(), start_y),
            (
                "Practice".to_string(),
                start_y + button_height + button_spacing,
            ),
            (
                "Analytics".to_string(),
                start_y + 2.0 * (button_height + button_spacing),
            ),
            (
                "Settings".to_string(),
                start_y + 3.0 * (button_height + button_spacing),
            ),
            (
                "Exit".to_string(),
                start_y + 4.0 * (button_height + button_spacing),
            ),
        ];

        menu_data.buttons.clear();
        for (label, y_pos) in &buttons {
            let button_x = (scr_width - button_width) / 2.0;
            menu_data.buttons.push((
                label.clone(),
                Rect::new(button_x, *y_pos, button_width, button_height),
            ));
        }
    }
}

fn exit_menu(mut commands: Commands) {
    commands.remove_resource::<MenuData>();
}

// ==================== SONG SELECTION STATE ====================

fn enter_song_selection(
    mut game_state: ResMut<GameStateResource>,
    mut selection_state: ResMut<SongSelectionState>,
) {
    game_state.songs = load_songs_from_assets();
    *selection_state = SongSelectionState::new();
}

fn update_song_selection(
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
}

// ==================== PRACTICE MENU STATE ====================

fn enter_practice_menu(
    mut game_state: ResMut<GameStateResource>,
    mut practice_state: ResMut<PracticeMenuState>,
) {
    game_state.songs = load_songs_from_assets();
    *practice_state = PracticeMenuState::new();
}

fn update_practice_menu(
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
}

// ==================== PLAYING STATE ====================

fn enter_playing(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Start beat detection in a new thread
    let song_path = game_state.selected_song.clone();

    commands.insert_resource(LoadingData {
        beats: None,
        start_time: Instant::now(),
        song_path: song_path.clone(),
    });

    // Spawn a thread to load beats
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let beats = gather_beats(&song_path);
        let _ = tx.send(beats);
    });

    // Store the receiver in a non-resource way using a channel
    // We'll check it in update_loading
    commands.insert_resource(BeatReceiver { rx: Some(rx) });

    // Transition to loading state
    next_state.set(AppState::Loading);
}

#[derive(Resource)]
struct BeatReceiver {
    rx: Option<mpsc::Receiver<Vec<f64>>>,
}

// ==================== LOADING STATE ====================

fn enter_loading() {
    // Loading screen setup is handled by setup_loading_ui
}

fn update_loading(
    mut commands: Commands,
    mut loading_data: ResMut<LoadingData>,
    mut beat_receiver: ResMut<BeatReceiver>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Check if beats are received
    if let Some(ref rx) = beat_receiver.rx {
        if let Ok(beats) = rx.try_recv() {
            loading_data.beats = Some(beats);

            commands.insert_resource(ReadyToPlayData {
                beats: loading_data.beats.clone().unwrap(),
                ready_time: Instant::now(),
            });

            commands.remove_resource::<LoadingData>();
            commands.remove_resource::<BeatReceiver>();
            next_state.set(AppState::ReadyToPlay);
        }
    }
}

// ==================== READY TO PLAY STATE ====================

fn enter_ready_to_play() {
    // Setup countdown
}

fn update_ready_to_play(
    mut commands: Commands,
    ready_data: Res<ReadyToPlayData>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio_sink: ResMut<GameAudioSink>,
    config: Res<GameConfig>,
    windows: Query<&Window>,
    game_state: Res<GameStateResource>,
) {
    let elapsed = ready_data.ready_time.elapsed().as_secs_f32();

    if elapsed >= COUNTDOWN_DURATION as f32 {
        // Load and start audio playback
        if let Ok(file) = std::fs::File::open(&game_state.selected_song) {
            let reader = std::io::BufReader::new(file);
            if let Ok(source) = Decoder::new(reader) {
                audio_sink.sink.append(source);
                audio_sink.sink.play();
            }
        }

        // Initialize visualization state
        if let Ok(window) = windows.single() {
            let width = window.width();
            let height = window.height();
            let mut rng = rand::thread_rng();

            let spawn_radius = calculate_spawn_radius(width, height);
            let center = Vec2::new(width / 2.0, height / 2.0);

            let circles = initialize_circles(
                &ready_data.beats,
                &mut rng,
                spawn_radius,
                center,
                SHRINK_TIME,
                COUNTDOWN_DURATION,
                &config,
            );

            let vis_state = VisualizingState::new(
                ready_data.beats.clone(),
                circles,
                config.clone(),
                game_state.selected_song.clone(),
            );

            commands.insert_resource(VisualizingData {
                state: vis_state,
                start_time: Instant::now(),
            });
        }

        commands.remove_resource::<ReadyToPlayData>();
        next_state.set(AppState::Visualizing);
    }
}

// ==================== VISUALIZING STATE ====================

fn enter_visualizing() {
    // Setup visualization
}

fn update_visualizing(
    mut visualizing_data: ResMut<VisualizingData>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio_sink: ResMut<GameAudioSink>,
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut analytics: ResMut<Analytics>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let base_elapsed = visualizing_data.start_time.elapsed().as_secs_f64();
    let elapsed = if visualizing_data.state.playback_speed != 1.0 {
        base_elapsed * visualizing_data.state.playback_speed as f64
    } else {
        base_elapsed
    };

    // Get mouse position for hit detection
    let mut mouse_pos = Vec2::ZERO;

    if let Ok(window) = windows.single() {
        if let Some(cursor_pos) = window.cursor_position() {
            mouse_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );
        }
    }

    // Check for key presses
    let key_pressed = keyboard.just_pressed(config.key_bindings.primary_hit_key())
        || keyboard.just_pressed(config.key_bindings.secondary_hit_key());

    // Handle key hits with mouse position
    if key_pressed {
        handle_key_hits_with_mouse(
            &mut visualizing_data.state.circles,
            elapsed,
            &mut visualizing_data.state,
            SHRINK_TIME,
            &config,
            mouse_pos,
        );
    }

    // Handle missed circles
    handle_missed_circles(
        &mut visualizing_data.state.circles,
        elapsed,
        &mut visualizing_data.state,
        SHRINK_TIME,
    );

    // Check for exit
    if keyboard.just_pressed(config.key_bindings.exit_key()) {
        audio_sink.sink.stop();

        if let Some(session) = visualizing_data.state.finish_session() {
            if config.save_analytics {
                analytics.add_session(session);
            }
        }

        next_state.set(AppState::Menu);
        return;
    }

    // Check if music has ended
    if audio_sink.sink.empty() {
        let active_session = visualizing_data.state.finish_session();

        let end_state = EndState {
            score: visualizing_data.state.score,
            max_combo: visualizing_data.state.max_combo,
            hits: if let Some(ref session) = active_session {
                session.hits.clone()
            } else {
                crate::analytics::HitStats::new()
            },
            accuracy: if let Some(ref session) = active_session {
                session.accuracy
            } else {
                0.0
            },
            grade: if let Some(ref session) = active_session {
                session.grade.clone()
            } else {
                crate::analytics::Grade::F
            },
            full_combo: visualizing_data.state.max_combo > 0
                && visualizing_data
                    .state
                    .circles
                    .iter()
                    .all(|c| c.hit || c.missed)
                && visualizing_data
                    .state
                    .circles
                    .iter()
                    .filter(|c| c.missed)
                    .count()
                    == 0,
            song_name: visualizing_data.state.song_name.clone(),
            practice_mode: visualizing_data.state.practice_mode,
            playback_speed: visualizing_data.state.playback_speed,
            new_best: false,
            previous_best: 0,
        };

        if config.save_analytics {
            if let Some(session) = active_session {
                analytics.add_session(session);
            }
        }

        commands.insert_resource(EndData { state: end_state });
        next_state.set(AppState::End);
    }
}

fn exit_visualizing(mut commands: Commands) {
    commands.remove_resource::<VisualizingData>();
}

// ==================== END STATE ====================

fn enter_end() {
    // Setup end screen
}

fn update_end(
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Menu);
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        next_state.set(AppState::Menu);
    }
}

// ==================== SETTINGS STATE ====================

fn enter_settings(mut settings_state: ResMut<SettingsState>) {
    *settings_state = SettingsState::new();
}

fn update_settings(
    mut next_state: ResMut<NextState<AppState>>,
    settings_state: ResMut<SettingsState>,
    mut config: ResMut<GameConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        config.save();
        next_state.set(AppState::Menu);
    }
}

// ==================== ANALYTICS STATE ====================

fn enter_analytics(mut analytics_state: ResMut<AnalyticsState>) {
    *analytics_state = AnalyticsState::new();
}

fn update_analytics(
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
}

// ==================== RENDERING SYSTEMS ====================

fn render_game_circles(mut commands: Commands, visualizing_data: Res<VisualizingData>) {
    let base_elapsed = visualizing_data.start_time.elapsed().as_secs_f64();
    let elapsed = if visualizing_data.state.playback_speed != 1.0 {
        base_elapsed * visualizing_data.state.playback_speed as f64
    } else {
        base_elapsed
    };

    draw_circles_bevy(
        &mut commands,
        &visualizing_data.state.circles,
        elapsed,
        SHRINK_TIME,
    );
}

fn render_game_floating_texts(
    mut commands: Commands,
    mut visualizing_data: ResMut<VisualizingData>,
    assets: Res<GameAssets>,
) {
    let base_elapsed = visualizing_data.start_time.elapsed().as_secs_f64();
    let elapsed = if visualizing_data.state.playback_speed != 1.0 {
        base_elapsed * visualizing_data.state.playback_speed as f64
    } else {
        base_elapsed
    };

    draw_floating_texts_bevy(
        &mut commands,
        &mut visualizing_data.state.floating_texts,
        elapsed,
        &assets,
    );
}

fn render_game_score(
    mut commands: Commands,
    visualizing_data: Res<VisualizingData>,
    assets: Res<GameAssets>,
) {
    draw_score_bevy(
        &mut commands,
        visualizing_data.state.score,
        visualizing_data.state.combo,
        visualizing_data.state.max_combo,
        &assets,
    );
}

/// Handle key hits with mouse position
fn handle_key_hits_with_mouse(
    circles: &mut Vec<structs::GameCircle>,
    elapsed: f64,
    vis_state: &mut VisualizingState,
    shrink_time: f64,
    config: &GameConfig,
    mouse_pos: Vec2,
) {
    // Find the closest hittable circle
    let mut best_circle_idx: Option<usize> = None;
    let mut best_distance = f32::MAX;

    for (idx, circle) in circles.iter().enumerate() {
        if circle.hit || circle.missed {
            continue;
        }

        if let Some(radius) = circle_radius(circle, elapsed, shrink_time) {
            let distance = mouse_pos.distance(circle.position);
            if distance < radius && distance < best_distance {
                best_distance = distance;
                best_circle_idx = Some(idx);
            }
        }
    }

    // Process the hit
    if let Some(idx) = best_circle_idx {
        let circle = &mut circles[idx];
        circle.hit = true;

        let hit_time_diff = (elapsed - circle.hit_time).abs();
        let points = calculate_score_from_timing(hit_time_diff);

        // Record the hit with timing
        let timing_ms = (hit_time_diff * 1000.0) as f32;
        vis_state.record_hit(points, timing_ms);

        // Add floating text
        let (text, color) = match points {
            300 => ("Perfect!", (0.0, 1.0, 0.5)),
            100 => ("Good!", (0.0, 0.75, 1.0)),
            50 => ("Okay", (1.0, 1.0, 0.0)),
            _ => ("Miss", (1.0, 0.0, 0.0)),
        };

        vis_state.floating_texts.push(FloatingText {
            text: text.to_string(),
            position: circle.position,
            spawn_time: elapsed,
            duration: 1.0,
            color,
        });
    }
}
