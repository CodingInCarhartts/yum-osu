mod constants;
mod structs;
mod audio;
mod ui;
mod game;

use crate::structs::*;
use crate::constants::*;
use crate::audio::*;
use crate::ui::*;
use crate::game::*;

use macroquad::prelude::*;
use rodio::{ Decoder, OutputStream, Sink };
use std::{ sync::mpsc, thread, time::Instant };

fn handle_menu_state(assets: &Assets, songs: &mut Vec<String>) -> GameState {
    if let Some(selected) = draw_menu(assets) {
        match selected.as_str() {
            "Start Game" => {
                *songs = load_songs_from_assets();
                GameState::SongSelection // Proceed to song selection
            }
            "Settings" => {
                GameState::Settings // Navigate to the settings screen (assuming you have this state)
            }
            "Exit" => {
                GameState::Exit // Assuming you have an exit state that handles closing the game
            }
            _ => GameState::Menu, // Default to the main menu if no match
        }
    } else {
        GameState::Menu // Stay in the menu if no button is clicked
    }
}

fn handle_song_selection_state(
    selected_song: &mut String,
    songs: &Vec<String>,
    assets: &Assets
) -> GameState {
    let mut selection_state = SongSelectionState::new();

    if let Some(song) = draw_choose_audio(&mut selection_state, songs, assets) {
        *selected_song = song;
        GameState::Playing
    } else {
        GameState::SongSelection
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
    assets: &Assets
) -> GameState {
    // Display the loading bar
    let loading_time = start_time.elapsed().as_secs_f32();
    draw_loading_bar(loading_time, assets);

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
    assets: &Assets
) -> GameState {
    // Display the countdown
    let elapsed = ready_time.elapsed().as_secs_f32();
    clear_background(DARK_BACKGROUND);
    if elapsed < (COUNTDOWN_DURATION as f32) {
        let scr_width = screen_width();
        let scr_height = screen_height();

        // Prepare the countdown text
        let countdown_text = format!("Starting in {:.0}", COUNTDOWN_DURATION - (elapsed as f64));

        // Measure the text dimensions to center it horizontally
        let text_dimensions = measure_text(
            &countdown_text,
            Some(&assets.cyberpunk_font),
            FONT_SIZE as u16,
            1.0
        );
        let text_x = (scr_width - text_dimensions.width) / 2.0; // Center horizontally
        let text_y = scr_height / 2.0; // Vertically centered

        // Draw the centered countdown text
        draw_text_ex(&countdown_text, text_x, text_y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: FONT_SIZE,
            color: NEON_GREEN,
            ..Default::default()
        });

        GameState::ReadyToPlay {
            beats,
            ready_time,
            source,
        }
    } else {
        // Start the audio playback
        if let Some(source) = source.take() {
            sink.append(source);
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
            COUNTDOWN_DURATION // Pass the delay here
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
    assets: &Assets
) -> GameState {
    // Visualization code
    let elapsed = vis_state.start_time.elapsed().as_secs_f64();

    clear_background(DARK_BACKGROUND);

    // Handle inputs, update circles, draw circles, etc.
    handle_key_hits(&mut vis_state.circles, elapsed, &mut vis_state.score, SHRINK_TIME);
    handle_missed_circles(
        &mut vis_state.circles,
        elapsed,
        &mut vis_state.floating_texts,
        SHRINK_TIME
    );
    draw_circles(&vis_state.circles, elapsed, SHRINK_TIME);
    draw_floating_texts(&mut vis_state.floating_texts, elapsed, assets);
    draw_score(vis_state.score, assets);

    if is_key_pressed(KeyCode::Escape) {
        // Optionally stop the music
        sink.stop();
        GameState::Menu
    } else if sink.empty() {
        // Music has ended
        GameState::End
    } else {
        GameState::Visualizing(vis_state)
    }
}

fn handle_end_state() -> GameState {
    // Clear the screen
    clear_background(BLACK);

    // Draw the game over text
    let scr_width = screen_width();
    let scr_height = screen_height();

    draw_text("Game Over", scr_width / 2.0 - 100.0, scr_height / 2.0 - 150.0, 40.0, WHITE);
    draw_text(
        "Press any key to return to the main menu.",
        scr_width / 2.0 - 200.0,
        scr_height / 2.0 - 100.0,
        30.0,
        WHITE
    );

    // Check if the user wants to quit
    if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
        GameState::Menu
    } else {
        GameState::End
    }
}

// Main game loop
#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::Menu;
    let mut selected_song = String::new();
    let mut songs = Vec::new();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();

    let assets = load_ui_assets().await;

    loop {
        state = match state {
            GameState::Menu => handle_menu_state(&assets, &mut songs),
            GameState::SongSelection =>
                handle_song_selection_state(&mut selected_song, &songs, &assets),
            GameState::Playing => handle_playing_state(&selected_song),
            GameState::Loading { rx, start_time } => {
                handle_loading_state(rx, start_time, &selected_song, &assets)
            }
            GameState::ReadyToPlay { beats, ready_time, source } => {
                handle_ready_to_play_state(beats, ready_time, source, &mut sink, &assets)
            }
            GameState::Visualizing(vis_state) =>
                handle_visualizing_state(vis_state, &mut sink, &assets),
            GameState::End => handle_end_state(),
            GameState::Settings => GameState::Settings,
            GameState::Exit => {
                break;
            }
        };

        next_frame().await;
    }
}
