mod constants;
mod structs;
mod audio;
mod ui;
mod game;
mod config;
mod analytics;
mod network;
mod accounts;
mod multiplayer;
mod community;

use crate::structs::*;
use crate::constants::*;
use crate::audio::*;
use crate::ui::*;
use crate::game::*;
use crate::config::{ GameConfig, SettingsState, KeyBindings };
use crate::analytics::{ Analytics, AnalyticsState };
use crate::network::GameClient;
use crate::accounts::AccountManager;
use crate::multiplayer::GameCoordinator;
use crate::community::CommunityManager;

use macroquad::prelude::*;
use rodio::{ Decoder, OutputStream, Sink };
use std::{ sync::mpsc, thread, time::Instant, sync::Arc };

fn handle_menu_state(
    assets: &Assets, 
    songs: &mut Vec<String>,
    config: &mut GameConfig
) -> GameState {
    if let Some(selected) = draw_menu(assets) {
        match selected.as_str() {
            "Start Game" => {
                *songs = load_songs_from_assets();
                GameState::SongSelection
            }
            "Practice" => {
                *songs = load_songs_from_assets();
                GameState::PracticeMenu
            }
            "Multiplayer" => {
                GameState::MultiplayerLobby
            }
            "Profile" => {
                GameState::Profile
            }
            "Leaderboard" => {
                GameState::Leaderboard
            }
            "Friends" => {
                GameState::Friends
            }
            "Community Hub" => {
                GameState::CommunityHub
            }
            "Analytics" => {
                GameState::Analytics
            }
            "Settings" => {
                GameState::Settings
            }
            "Exit" => {
                GameState::Exit
            }
            _ => GameState::Menu,
        }
    } else {
        GameState::Menu
    }
}

fn handle_song_selection_state(
    selected_song: &mut String,
    songs: &Vec<String>,
    assets: &Assets,
    config: &mut GameConfig
) -> GameState {
    let mut selection_state = SongSelectionState::new();

    if let Some(song) = draw_choose_audio(&mut selection_state, 
        songs, 
        assets
    ) {
        *selected_song = song;
        GameState::Playing
    } else if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else {
        GameState::SongSelection
    }
}

fn handle_practice_menu_state(
    practice_state: &mut PracticeMenuState,
    songs: &Vec<String>,
    assets: &Assets,
    config: &mut GameConfig
) -> GameState {
    match draw_practice_menu(practice_state, songs, assets) {
        Some(action) => {
            if action == "start" {
                if let Some(ref song) = practice_state.selected_song {
                    config.practice.playback_speed = practice_state.playback_speed;
                    config.practice.no_fail = practice_state.no_fail;
                    config.practice.autoplay = practice_state.autoplay;
                    config.practice.hit_sounds = practice_state.hit_sounds;
                    GameState::Playing
                } else {
                    GameState::PracticeMenu
                }
            } else if action == "back" {
                GameState::Menu
            } else {
                GameState::PracticeMenu
            }
        }
        None => GameState::PracticeMenu,
    }
}

fn handle_playing_state(selected_song: &String) -> GameState {
    // Start the beat detection in a new thread
    let (tx, rx) = mpsc::channel();
    let song_path = selected_song.clone();
    thread::spawn(move || {
        let beats = gather_beats(&song_path);
        tx.send(beats).unwrap();
    });

    // Switch to the loading state
    GameState::Loading {
        rx,
        start_time: Instant::now(),
    }
}

fn handle_loading_state(
    rx: mpsc::Receiver<Vec<f64>>,
    start_time: Instant,
    selected_song: &String,
    assets: &Assets,
    config: &GameConfig
) -> GameState {
    // Display the loading bar
    let loading_time = start_time.elapsed().as_secs_f32();
    let message = if config.practice.playback_speed != 1.0 {
        Some(&format!("Loading... ({:.2}x speed)", config.practice.playback_speed))
    } else {
        None
    };
    draw_loading_bar(loading_time, assets, message.map(|s| s.as_str()));

    // Check if the beats are received
    if let Ok(beats) = rx.try_recv() {
        // Load the audio file but don't play it yet
        let file = std::fs::File::open(selected_song).expect("Failed to open audio file");
        let reader = std::io::BufReader::new(file);
        let source = Decoder::new(reader).expect("Failed to decode audio");

        // Switch to the ready to play state
        GameState::ReadyToPlay {
            beats,
            ready_time: Instant::now(),
            source: Some(source),
        }
    } else {
        // Stay in the loading state
        GameState::Loading {
            rx,
            start_time,
        }
    }
}

fn handle_ready_to_play_state(
    beats: Vec<f64>,
    ready_time: Instant,
    mut source: Option<Decoder<std::io::BufReader<std::fs::File>>>,
    sink: &mut Sink,
    assets: &Assets,
    config: &GameConfig,
    song_name: &str
) -> GameState {
    // Display the countdown
    let elapsed = ready_time.elapsed().as_secs_f32();
    clear_background(DARK_BACKGROUND);
    if elapsed < (COUNTDOWN_DURATION as f32) {
        let scr_width = screen_width();
        let scr_height = screen_height();

        // Draw countdown
        let countdown_text = format!("Starting in {:.0}", COUNTDOWN_DURATION - (elapsed as f64));

        let text_dimensions = measure_text(
            &countdown_text,
            Some(&assets.cyberpunk_font),
            FONT_SIZE as u16,
            1.0
        );
        let text_x = (scr_width - text_dimensions.width) / 2.0;
        let text_y = scr_height / 2.0;

        draw_text_ex(&countdown_text, text_x, text_y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: FONT_SIZE,
            color: NEON_GREEN,
            ..Default::default()
        });

        // Show practice mode info if active
        if config.practice.playback_speed != 1.0 || config.practice.no_fail {
            let practice_info = format!(
                "Practice: {:.2}x speed {}{}",
                config.practice.playback_speed,
                if config.practice.no_fail { "| No-Fail " } else { "" },
                if config.practice.autoplay { "| Autoplay" } else { "" }
            );
            let info_dim = measure_text(&practice_info,
                Some(&assets.cyberpunk_font),
                20,
                1.0
            );
            draw_text_ex(&practice_info, 
                (scr_width - info_dim.width) / 2.0, 
                text_y + 50.0, 
                TextParams {
                    font: Some(&assets.cyberpunk_font),
                    font_size: 20,
                    color: NEON_YELLOW,
                    ..Default::default()
                }
            );
        }

        GameState::ReadyToPlay {
            beats,
            ready_time,
            source,
        }
    } else {
        // Start the audio playback
        if let Some(source) = source.take() {
            // Apply playback speed if needed
            let speed = config.practice.playback_speed;
            if speed != 1.0 {
                // Note: rodio speed modification would require additional implementation
                sink.append(source);
            } else {
                sink.append(source);
            }
            sink.play();
        }

        // Initialize the visualization state
        let (width, height) = (screen_width(), screen_height());
        let mut rng = ::rand::thread_rng();

        let spawn_radius = calculate_spawn_radius(width, height);
        let center = Vec2::new(width / 2.0, height / 2.0);

        let circles = initialize_circles(
            &beats,
            &mut rng,
            spawn_radius,
            center,
            SHRINK_TIME,
            COUNTDOWN_DURATION,
            config
        );

        let vis_state = VisualizingState::new(
            beats.clone(),
            circles,
            config.clone(),
            song_name.to_string()
        );
        let score = 0;
        let floating_texts = Vec::with_capacity(10); // Pre-allocate with reasonable capacity

        GameState::Visualizing(
            Box::new(VisualizingState {
                beats,
                start_time: Instant::now(),
                circles,
                score,
                floating_texts,
            })
        )
    }
}

fn handle_visualizing_state(
    mut vis_state: Box<VisualizingState>,
    sink: &mut Sink,
    assets: &Assets,
    config: &GameConfig,
    analytics: &mut Analytics
) -> GameState {
    // Adjust elapsed time for playback speed
    let base_elapsed = vis_state.start_time.elapsed().as_secs_f64();
    let elapsed = if vis_state.playback_speed != 1.0 {
        base_elapsed * vis_state.playback_speed as f64
    } else {
        base_elapsed
    };

    clear_background(DARK_BACKGROUND);

    // Handle inputs, update circles, draw circles, etc.
    handle_key_hits(
        &mut vis_state.circles, 
        elapsed, 
        &mut vis_state, 
        SHRINK_TIME,
        config
    );
    
    handle_missed_circles(
        &mut vis_state.circles,
        elapsed,
        &mut vis_state,
        SHRINK_TIME
    );
    
    draw_circles(&vis_state.circles, 
        elapsed, 
        SHRINK_TIME,
        config
    );
    
    draw_floating_texts(&mut vis_state.floating_texts, 
        elapsed, 
        assets
    );
    
    draw_score(
        vis_state.score, 
        vis_state.combo,
        vis_state.max_combo,
        assets
    );

    // Draw practice mode indicators
    if vis_state.practice_mode {
        let scr_width = screen_width();
        let practice_text = format!("Practice: {:.2}x", vis_state.playback_speed);
        draw_text_ex(&practice_text, 
            scr_width - 150.0, 
            20.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 16,
                color: NEON_YELLOW,
                ..Default::default()
            }
        );

        if vis_state.no_fail {
            draw_text_ex("NO-FAIL", scr_width - 150.0, 40.0, TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 14,
                color: NEON_GREEN,
                ..Default::default()
            });
        }
    }

    // Check for exit
    if is_key_pressed(config.key_bindings.exit_key()) {
        sink.stop();
        
        // Save analytics if enabled
        if let Some(session) = vis_state.finish_session() {
            if config.save_analytics {
                analytics.add_session(session);
            }
        }
        
        return GameState::Menu;
    }

    // Check if music has ended
    if sink.empty() {
        // Create end state
        let active_session = vis_state.finish_session();
        
        let end_state = EndState {
            score: vis_state.score,
            max_combo: vis_state.max_combo,
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
            full_combo: vis_state.max_combo > 0 && 
                vis_state.circles.iter().all(|c| c.hit || c.missed) &&
                vis_state.circles.iter().filter(|c| c.missed).count() == 0,
            song_name: vis_state.song_name.clone(),
            practice_mode: vis_state.practice_mode,
            playback_speed: vis_state.playback_speed,
            new_best: false, // Will be set later
            previous_best: 0,
        };

        // Save analytics
        if config.save_analytics {
            if let Some(session) = active_session {
                analytics.add_session(session);
            }
        }

        return GameState::End(Box::new(end_state));
    }

    GameState::Visualizing(vis_state)
}

fn handle_end_state(
    end_state: Box<EndState>,
    assets: &Assets
) -> GameState {
    match draw_end_screen(&end_state,
        assets
    ) {
        Some(_) => GameState::Menu,
        None => GameState::End(end_state),
    }
}

fn handle_settings_state(
    settings_state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets
) -> GameState {
    match draw_settings(settings_state, config, assets) {
        Some(action) => {
            if action == "back" || action == "saved" {
                GameState::Menu
            } else {
                GameState::Settings
            }
        }
        None => GameState::Settings,
    }
}

fn handle_analytics_state(
    analytics_state: &mut AnalyticsState,
    analytics: &Analytics,
    assets: &Assets
) -> GameState {
    match draw_analytics(analytics_state, analytics, assets) {
        Some(action) => {
            if action == "back" {
                GameState::Menu
            } else {
                GameState::Analytics
            }
        }
        None => GameState::Analytics,
    }
}

// Handler for login state
fn handle_login_state(
    login_state: &mut LoginState,
    assets: &Assets,
    account_manager: &Arc<AccountManager>,
    user_session: &mut Option<UserSession>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    // Draw login UI
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Login";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 60, 1.0).width) / 2.0,
        screen_height * 0.2,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 60,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Username field
    draw_text_ex(&format!("Username: {}", login_state.username),
        screen_width * 0.3,
        screen_height * 0.4,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 30,
            color: WHITE,
            ..Default::default()
        }
    );

    // Password field (masked)
    let password_display = "*".repeat(login_state.password.len());
    draw_text_ex(&format!("Password: {}", password_display),
        screen_width * 0.3,
        screen_height * 0.5,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 30,
            color: WHITE,
            ..Default::default()
        }
    );

    // Error message
    if let Some(ref error) = login_state.error_message {
        draw_text_ex(error,
            (screen_width - measure_text(error, Some(&assets.cyberpunk_font), 24, 1.0).width) / 2.0,
            screen_height * 0.6,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 24,
                color: NEON_RED,
                ..Default::default()
            }
        );
    }

    // Instructions
    draw_text_ex("Press ENTER to login, R to register, ESC to back to menu",
        (screen_width - measure_text("Press ENTER to login, R to register, ESC to back to menu",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.8,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    // Handle input
    if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else if is_key_pressed(KeyCode::Enter) && !login_state.username.is_empty() && !login_state.password.is_empty() {
        // Attempt login
        // For demo, we'll create a session directly
        *user_session = Some(UserSession::new(
            uuid::Uuid::new_v4(),
            login_state.username.clone(),
            "demo_token".to_string()
        ));
        GameState::Menu
    } else if is_key_pressed(KeyCode::R) {
        GameState::Register
    } else {
        GameState::Login
    }
}

// Handler for register state
fn handle_register_state(
    register_state: &mut RegisterState,
    assets: &Assets,
    account_manager: &Arc<AccountManager>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Register";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 60, 1.0).width) / 2.0,
        screen_height * 0.15,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 60,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Fields
    draw_text_ex(&format!("Username: {}", register_state.username),
        screen_width * 0.3,
        screen_height * 0.3,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex(&format!("Email: {}", register_state.email),
        screen_width * 0.3,
        screen_height * 0.4,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    let password_display = "*".repeat(register_state.password.len());
    draw_text_ex(&format!("Password: {}", password_display),
        screen_width * 0.3,
        screen_height * 0.5,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    let confirm_display = "*".repeat(register_state.confirm_password.len());
    draw_text_ex(&format!("Confirm Password: {}", confirm_display),
        screen_width * 0.3,
        screen_height * 0.6,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    // Error message
    if let Some(ref error) = register_state.error_message {
        draw_text_ex(error,
            (screen_width - measure_text(error, Some(&assets.cyberpunk_font), 24, 1.0).width) / 2.0,
            screen_height * 0.7,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 24,
                color: NEON_RED,
                ..Default::default()
            }
        );
    }

    // Instructions
    draw_text_ex("Press ENTER to register, ESC to back",
        (screen_width - measure_text("Press ENTER to register, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.85,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::Login
    } else {
        GameState::Register
    }
}

// Handler for multiplayer lobby state
fn handle_multiplayer_lobby_state(
    lobby_state: &mut MultiplayerLobbyState,
    assets: &Assets,
    game_client: &GameClient,
    user_session: &Option<UserSession>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Multiplayer Lobby";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 50, 1.0).width) / 2.0,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 50,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Show connected status
    let status = if user_session.is_some() { "Connected" } else { "Not Logged In" };
    draw_text_ex(status,
        screen_width * 0.8,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: if user_session.is_some() { NEON_GREEN } else { NEON_RED },
            ..Default::default()
        }
    );

    // Placeholder room list
    draw_text_ex("Available Rooms (Demo):",
        screen_width * 0.1,
        screen_height * 0.25,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 30,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    draw_text_ex("  Room 1 - 2/4 players - Hosted by Player1",
        screen_width * 0.1,
        screen_height * 0.32,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex("  Room 2 - 3/4 players - Hosted by Player2",
        screen_width * 0.1,
        screen_height * 0.38,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    // Instructions
    draw_text_ex("Press C to create room, J to join, ESC to back",
        (screen_width - measure_text("Press C to create room, J to join, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.9,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else {
        GameState::MultiplayerLobby
    }
}

// Handler for profile state
fn handle_profile_state(
    profile_state: &mut ProfileState,
    assets: &Assets,
    account_manager: &Arc<AccountManager>,
    user_session: &Option<UserSession>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Profile";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 50, 1.0).width) / 2.0,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 50,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Show user info
    if let Some(session) = user_session {
        draw_text_ex(&format!("Username: {}", session.username),
            screen_width * 0.1,
            screen_height * 0.3,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 30,
                color: WHITE,
                ..Default::default()
            }
        );
    } else {
        draw_text_ex("Not logged in",
            screen_width * 0.1,
            screen_height * 0.3,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 30,
                color: NEON_RED,
                ..Default::default()
            }
        );
    }

    // Tabs
    draw_text_ex("[Overview] [Stats] [Achievements] [Scores]",
        (screen_width - measure_text("[Overview] [Stats] [Achievements] [Scores]",
            Some(&assets.cyberpunk_font), 24, 1.0).width) / 2.0,
        screen_height * 0.5,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    draw_text_ex("Press TAB to switch tabs, ESC to back",
        (screen_width - measure_text("Press TAB to switch tabs, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.9,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else {
        GameState::Profile
    }
}

// Handler for leaderboard state
fn handle_leaderboard_state(
    leaderboard_state: &mut LeaderboardState,
    assets: &Assets,
    account_manager: &Arc<AccountManager>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Leaderboard";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 50, 1.0).width) / 2.0,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 50,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Tabs
    draw_text_ex("[Global] [Country] [Friends]",
        (screen_width - measure_text("[Global] [Country] [Friends]",
            Some(&assets.cyberpunk_font), 24, 1.0).width) / 2.0,
        screen_height * 0.2,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    // Placeholder leaderboard
    draw_text_ex("Rank  |  Player        |  Score      |  Accuracy",
        screen_width * 0.1,
        screen_height * 0.35,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    draw_text_ex("  1   |  ProPlayer     |  9,999,999  |  99.9%",
        screen_width * 0.1,
        screen_height * 0.42,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_ORANGE,
            ..Default::default()
        }
    );

    draw_text_ex("  2   |  MasterRhythm  |  8,888,888  |  99.5%",
        screen_width * 0.1,
        screen_height * 0.49,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex("  3   |  BeatMaster    |  7,777,777  |  99.2%",
        screen_width * 0.1,
        screen_height * 0.56,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex("Press TAB to switch tabs, ESC to back",
        (screen_width - measure_text("Press TAB to switch tabs, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.9,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else {
        GameState::Leaderboard
    }
}

// Handler for friends state
fn handle_friends_state(
    friends_state: &mut FriendsState,
    assets: &Assets,
    account_manager: &Arc<AccountManager>,
    user_session: &Option<UserSession>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Friends";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 50, 1.0).width) / 2.0,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 50,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Placeholder friends list
    draw_text_ex("Friends List:",
        screen_width * 0.1,
        screen_height * 0.25,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 30,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    draw_text_ex("  Friend1 - Online - Playing: Song.mp3",
        screen_width * 0.1,
        screen_height * 0.32,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: NEON_GREEN,
            ..Default::default()
        }
    );

    draw_text_ex("  Friend2 - Offline - Last seen: 2h ago",
        screen_width * 0.1,
        screen_height * 0.38,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: GRAY,
            ..Default::default()
        }
    );

    draw_text_ex("Press F to find friends, ESC to back",
        (screen_width - measure_text("Press F to find friends, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.9,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else {
        GameState::Friends
    }
}

// Handler for community hub state
fn handle_community_hub_state(
    community_hub_state: &mut CommunityHubState,
    assets: &Assets,
    community_manager: &Arc<CommunityManager>,
    user_session: &Option<UserSession>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Community Hub";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 50, 1.0).width) / 2.0,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 50,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Tabs
    draw_text_ex("[Tournaments] [Chat] [Events]",
        (screen_width - measure_text("[Tournaments] [Chat] [Events]",
            Some(&assets.cyberpunk_font), 24, 1.0).width) / 2.0,
        screen_height * 0.25,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    // Placeholder tournaments
    draw_text_ex("Active Tournaments:",
        screen_width * 0.1,
        screen_height * 0.4,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 30,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    draw_text_ex("  Weekly Championship - 32/64 players - Prize: 1000 pts",
        screen_width * 0.1,
        screen_height * 0.47,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex("  Monthly Showdown - Registration Open",
        screen_width * 0.1,
        screen_height * 0.54,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex("Press TAB to switch tabs, ESC to back",
        (screen_width - measure_text("Press TAB to switch tabs, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.9,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::Menu
    } else {
        GameState::CommunityHub
    }
}

// Handler for tournament state
fn handle_tournament_state(
    tournament_state: &mut TournamentState,
    assets: &Assets,
    community_manager: &Arc<CommunityManager>,
    user_session: &Option<UserSession>
) -> GameState {
    clear_background(DARK_BACKGROUND);

    let screen_width = screen_width();
    let screen_height = screen_height();

    // Title
    let title = "Tournament Details";
    draw_text_ex(title,
        (screen_width - measure_text(title, Some(&assets.cyberpunk_font), 50, 1.0).width) / 2.0,
        screen_height * 0.1,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 50,
            color: NEON_CYAN,
            ..Default::default()
        }
    );

    // Placeholder tournament details
    draw_text_ex("Tournament: Weekly Championship",
        screen_width * 0.1,
        screen_height * 0.3,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 30,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    draw_text_ex("Status: Registration Open",
        screen_width * 0.1,
        screen_height * 0.37,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: NEON_GREEN,
            ..Default::default()
        }
    );

    draw_text_ex("Players: 32/64",
        screen_width * 0.1,
        screen_height * 0.44,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: WHITE,
            ..Default::default()
        }
    );

    draw_text_ex("Prize: 1000 points",
        screen_width * 0.1,
        screen_height * 0.51,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: NEON_ORANGE,
            ..Default::default()
        }
    );

    draw_text_ex("Press ENTER to join, ESC to back",
        (screen_width - measure_text("Press ENTER to join, ESC to back",
            Some(&assets.cyberpunk_font), 20, 1.0).width) / 2.0,
        screen_height * 0.9,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        }
    );

    if is_key_pressed(KeyCode::Escape) {
        GameState::CommunityHub
    } else {
        GameState::Tournament
    }
}

// Main game loop
#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::Menu;
    let mut selected_song = String::new();
    let mut songs = Vec::new();

    // Load or create configuration
    let mut config = GameConfig::load();
    
    // Load analytics
    let mut analytics = Analytics::load();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();

    let assets = load_ui_assets().await;

    // State for new screens
    let mut settings_state = SettingsState::new();
    let mut analytics_state = AnalyticsState::new();
    let mut practice_state = PracticeMenuState::new();

    // Multiplayer and account state
    let mut login_state = LoginState::new();
    let mut register_state = RegisterState::new();
    let mut multiplayer_lobby_state = MultiplayerLobbyState::new();
    let mut profile_state = ProfileState::new();
    let mut leaderboard_state = LeaderboardState::new();
    let mut friends_state = FriendsState::new();
    let mut community_hub_state = CommunityHubState::new();
    let mut tournament_state = TournamentState::new();

    // Multiplayer and account managers
    let game_client = GameClient::new();
    let account_manager = Arc::new(AccountManager::new(std::path::PathBuf::from("data")));
    let game_coordinator = Arc::new(GameCoordinator::new());
    let community_manager = Arc::new(CommunityManager::new());

    // Load account data
    let _ = account_manager.load_data();

    // User session (will be populated after login)
    let mut user_session: Option<UserSession> = None;

    loop {
        state = match state {
            // Multiplayer and account states
            GameState::Login => handle_login_state(&mut login_state, &assets, &account_manager, &mut user_session),

            GameState::Register => handle_register_state(&mut register_state, &assets, &account_manager),

            GameState::MultiplayerLobby => handle_multiplayer_lobby_state(
                &mut multiplayer_lobby_state,
                &assets,
                &game_client,
                &user_session
            ),

            GameState::Profile => handle_profile_state(&mut profile_state, &assets, &account_manager, &user_session),

            GameState::Leaderboard => handle_leaderboard_state(&mut leaderboard_state, &assets, &account_manager),

            GameState::Friends => handle_friends_state(&mut friends_state, &assets, &account_manager, &user_session),

            GameState::CommunityHub => handle_community_hub_state(&mut community_hub_state, &assets, &community_manager, &user_session),

            GameState::Tournament => handle_tournament_state(&mut tournament_state, &assets, &community_manager, &user_session),

            GameState::Menu => handle_menu_state(&assets, &mut songs, &mut config),
            
            GameState::SongSelection => handle_song_selection_state(
                &mut selected_song, 
                &songs, 
                &assets,
                &mut config
            ),
            
            GameState::PracticeMenu => handle_practice_menu_state(
                &mut practice_state,
                &songs,
                &assets,
                &mut config
            ),
            
            GameState::Playing => handle_playing_state(&selected_song),
            
            GameState::Loading { rx, start_time } => {
                handle_loading_state(
                    rx, 
                    start_time, 
                    &selected_song, 
                    &assets,
                    &config
                )
            }
            
            GameState::ReadyToPlay { beats, ready_time, source } => {
                handle_ready_to_play_state(
                    beats, 
                    ready_time, 
                    source, 
                    &mut sink, 
                    &assets,
                    &config,
                    &selected_song
                )
            }
            
            GameState::Visualizing(vis_state) => handle_visualizing_state(
                vis_state, 
                &mut sink, 
                &assets,
                &config,
                &mut analytics
            ),
            
            GameState::End(end_state) => handle_end_state(end_state, &assets),
            
            GameState::Settings => handle_settings_state(
                &mut settings_state, 
                &mut config, 
                &assets
            ),
            
            GameState::Analytics => handle_analytics_state(
                &mut analytics_state, 
                &analytics, 
                &assets
            ),
            
            GameState::Exit => {
                // Save before exit
                config.save();
                analytics.save();
                break;
            }
        };

        next_frame().await;
    }
}
