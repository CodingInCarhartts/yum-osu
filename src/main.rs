mod constants;
mod structs;
mod audio;
mod ui;
mod game;
mod config;
mod analytics;

use crate::structs::*;
use crate::constants::*;
use crate::audio::*;
use crate::ui::*;
use crate::game::*;
use crate::config::{ GameConfig, SettingsState, KeyBindings };
use crate::analytics::{ Analytics, AnalyticsState };

use macroquad::prelude::*;
use rodio::{ Decoder, OutputStream, Sink };
use std::{ sync::mpsc, thread, time::Instant };

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

    loop {
        state = match state {
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
