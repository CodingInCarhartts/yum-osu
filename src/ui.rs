use macroquad::{
    color::WHITE,
    input::{ is_key_down, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton },
    prelude::Color,
    shapes::{ draw_line, draw_rectangle, draw_rectangle_lines },
    text::{ draw_text_ex, load_ttf_font, measure_text, TextParams },
    time::get_time,
    window::{ clear_background, screen_height, screen_width },
};
use crate::structs::{ Assets, SongSelectionState, FloatingText };
use crate::constants::*;
use std::fs;

/// Load all UI assets, such as textures and fonts.
///
/// UI assets are loaded asynchronously, so this function returns a future.
/// The future resolves to an `Assets` struct containing all the loaded assets.
///
/// The assets are loaded from the `src/assets/images/` directory.
///
/// * `menu_background` is the background image for the main menu.
/// * `start_button` is the start button image.
pub async fn load_ui_assets() -> Assets {
    let cyberpunk_font = load_ttf_font("src/assets/fonts/teknaf.otf").await.unwrap();

    Assets {
        cyberpunk_font,
    }
}

/// Draw the main menu.
///
/// The main menu is drawn in the center of the screen and consists of a background image,
/// a start button, and a quit button.
///
/// The `assets` parameter is an `Assets` struct containing the loaded UI assets.
///
/// The `start_game` parameter is a boolean indicating whether the player has pressed the start button.
///
/// If the player has pressed the start button, the function returns `true`.
///
/// If the player has not pressed the start button, the function returns `false`.
///
/// The function also draws the UI elements, such as the background image and the start button.
pub fn draw_menu(assets: &Assets) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let scr_width = screen_width();
    let scr_height = screen_height();

    // Set the animated gradient background (optional)
    let elapsed = get_time(); // Assuming this tracks elapsed time
    //draw_background(scr_width, scr_height, elapsed);

    // Draw the title with neon glow
    let title_text = "YumOsu!";
    let font_size = 72.0;
    let text_dimensions = measure_text(
        title_text,
        Some(&assets.cyberpunk_font),
        font_size as u16,
        1.0
    );
    let text_x = (scr_width - text_dimensions.width) / 2.0;
    let text_y = scr_height * 0.2; // Place the title at 20% of screen height

    // Draw glowing title text
    draw_text_ex(title_text, text_x, text_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: font_size as u16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Button properties
    let button_width = 200.0;
    let button_height = 60.0;
    let button_spacing = scr_height * 0.05; // 5% of screen height as spacing

    // Calculate starting Y position for the buttons (start at 40% of the screen height)
    let start_y = scr_height * 0.4;

    // Create a vector of buttons with labels and corresponding y-positions
    let buttons = vec![
        ("Start Game", start_y),
        ("Settings", start_y + button_height + button_spacing),
        ("Exit", start_y + 2.0 * (button_height + button_spacing))
    ];

    // Loop through buttons and draw them
    let mut selected_button: Option<String> = None;
    for (label, y_pos) in buttons.iter() {
        let button_x = (scr_width - button_width) / 2.0;

        // Check if the button is hovered
        let mouse_pos = mouse_position();
        let is_hovered =
            mouse_pos.0 >= button_x &&
            mouse_pos.0 <= button_x + button_width &&
            mouse_pos.1 >= *y_pos &&
            mouse_pos.1 <= *y_pos + button_height;

        // Change color when hovered
        let button_color = if is_hovered { NEON_GREEN } else { NEON_BLUE };

        draw_rectangle(button_x, *y_pos, button_width, button_height, button_color);

        // Add glow effect around the button - reduced iterations for performance
        for i in 1..3 {
            let glow_alpha = 0.15 / (i as f32);
            draw_rectangle_lines(
                button_x - (i as f32),
                *y_pos - (i as f32),
                button_width + 2.0 * (i as f32),
                button_height + 2.0 * (i as f32),
                2.0,
                Color::new(button_color.r, button_color.g, button_color.b, glow_alpha)
            );
        }

        // Draw the button text
        let text_dimensions = measure_text(
            label,
            Some(&assets.cyberpunk_font),
            CYBERPUNK_FONT_SIZE as u16,
            1.0
        );
        let text_x = button_x + (button_width - text_dimensions.width) / 2.0;
        let text_y = y_pos + (button_height + text_dimensions.height) / 2.0;

        draw_text_ex(label, text_x, text_y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: CYBERPUNK_FONT_SIZE as u16,
            color: WHITE,
            ..Default::default()
        });

        // Check if the button is clicked
        if is_mouse_button_pressed(MouseButton::Left) && is_hovered {
            selected_button = Some(label.to_string());
        }
    }

    // Return the selected button label if clicked, else None
    selected_button
}

/// Implement the `SongSelectionState` struct.
///
/// The `SongSelectionState` struct is used to store the state of the song selection menu.
///
/// The `new` function creates a new `SongSelectionState` instance with default values.
impl SongSelectionState {
    pub fn new() -> Self {
        Self {
            scroll_pos: 0.0,
            selected_song: None,
        }
    }
}

/// Draw the song selection menu.
///
/// The song selection menu is drawn in the center of the screen and consists of a list of songs.
///
/// The `state` parameter is a `SongSelectionState` struct containing the state of the song selection menu.
///
/// The `songs` parameter is a vector of strings containing the names of all the songs.
///
/// The `selected_song` parameter is an `Option<String>` containing the selected song, if any.
///
/// If the player has selected a song, the function returns `Some(song)`, where `song` is the selected song.
///
/// If the player has not selected a song, the function returns `None`.
///
/// The function also draws the UI elements, such as the list of songs and the selected song.
pub fn draw_choose_audio(
    state: &mut SongSelectionState,
    songs: &[String],
    assets: &Assets
) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Get the time for animations (pulsing glow, etc.)
    let elapsed_time = get_time();

    // Draw the title at the top
    let title_text = "Select a Song";
    draw_text_ex(title_text, 20.0, screen_h * 0.1, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: CYBERPUNK_FONT_SIZE as u16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Handle scrolling with Up/Down arrow keys
    if is_key_down(KeyCode::Down) {
        state.scroll_pos += 5.0;
    }
    if is_key_down(KeyCode::Up) {
        state.scroll_pos -= 5.0;
    }

    // Clamp scroll position to prevent overscrolling
    let max_scroll = (songs.len() as f32) * (SONG_ENTRY_HEIGHT + 20.0) - screen_h * 0.7;
    state.scroll_pos = state.scroll_pos.clamp(0.0, max_scroll.max(0.0));

    // Define the vertical gap between song entries
    let vertical_gap = 20.0;

    // Iterate through the songs and draw them as buttons
    for (i, song) in songs.iter().enumerate() {
        let button_x = screen_w * 0.05; // 5% from the left edge
        let button_y =
            screen_h * 0.2 + (i as f32) * (SONG_ENTRY_HEIGHT + vertical_gap) - state.scroll_pos;

        if button_y > SONG_ENTRY_HEIGHT && button_y < screen_h - SONG_ENTRY_HEIGHT {
            let button_width = screen_w * 0.9; // 90% of screen width
            let button_height = SONG_ENTRY_HEIGHT;

            // Check if the button is hovered
            let mouse_pos = mouse_position();
            let is_hovered =
                mouse_pos.0 >= button_x &&
                mouse_pos.0 <= button_x + button_width &&
                mouse_pos.1 >= button_y &&
                mouse_pos.1 <= button_y + button_height;

            // Hover animation: Scale the button when hovered
            let scale_factor = if is_hovered { 1.1 } else { 1.0 };
            let scaled_button_width = button_width * scale_factor;
            let scaled_button_height = button_height * scale_factor;
            let scaled_button_x = button_x - (scaled_button_width - button_width) / 2.0;
            let scaled_button_y = button_y - (scaled_button_height - button_height) / 2.0;

            // Glow animation: Pulse the glow
            let pulse_intensity = 0.5 + (elapsed_time.sin() as f32) * 0.5;
            let glow_color = Color::new(NEON_GREEN.r, NEON_GREEN.g, NEON_GREEN.b, pulse_intensity);

            // Draw neon rectangle for the song entry with scaling
            draw_rectangle(
                scaled_button_x,
                scaled_button_y,
                scaled_button_width,
                scaled_button_height,
                NEON_BLUE
            );

            // Add pulsing glow effect around the button - only when hovered for performance
            if is_hovered {
                for glow_level in 1..3 {
                    let glow_alpha = (0.1 / (glow_level as f32)) * pulse_intensity;
                    draw_rectangle_lines(
                        scaled_button_x - (glow_level as f32),
                        scaled_button_y - (glow_level as f32),
                        scaled_button_width + 2.0 * (glow_level as f32),
                        scaled_button_height + 2.0 * (glow_level as f32),
                        1.0,
                        Color::new(glow_color.r, glow_color.g, glow_color.b, glow_alpha)
                    );
                }
            }

            // Extract the song name (last part of the path) and remove .mp3 from the end of it
            let song_name = song
                .split('/')
                .last()
                .unwrap_or(song)
                .to_uppercase()
                .replace(".MP3", "");

            // Measure text to center it within the scaled button
            let text_dimensions = measure_text(
                &song_name,
                Some(&assets.cyberpunk_font),
                CYBERPUNK_FONT_SIZE as u16,
                1.0
            );
            let text_x = scaled_button_x + (scaled_button_width - text_dimensions.width) / 2.0; // Center horizontally
            let text_y = scaled_button_y + (scaled_button_height + text_dimensions.height) / 2.0; // Center vertically

            // Draw the song name centered on the scaled button
            draw_text_ex(&song_name, text_x, text_y, TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: CYBERPUNK_FONT_SIZE as u16,
                color: WHITE,
                ..Default::default()
            });

            // Check if the song entry is clicked
            if is_mouse_button_pressed(MouseButton::Left) && is_hovered {
                return Some(song.clone());
            }
        }
    }

    None
}

/// Load all songs from the assets directory.
///
/// The function reads the `src/assets/music/` directory and loads all the songs with the `.mp3` extension.
///
/// The function returns a vector of strings containing the names of all the songs.
pub fn load_songs_from_assets() -> Vec<String> {
    let mut songs = Vec::new();
    if let Ok(entries) = fs::read_dir("src/assets/music/") {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "mp3" {
                    let full_path = entry.path().to_string_lossy().to_string();
                    songs.push(full_path.clone());
                    println!("Loaded song: {}", full_path.clone());
                }
            }
        }
    }
    songs
}

/// Draw a loading bar.
///
/// The loading bar is drawn in the center of the screen.
///
/// The `elapsed_time` parameter is the elapsed time since the loading started.
///
/// The function draws a black rectangle with a white border and a loading bar inside.
///
/// The loading bar is a gray rectangle that fills up the black rectangle as the elapsed time increases.
pub fn draw_loading_bar(elapsed_time: f32, assets: &Assets) {
    let scr_width = screen_width();
    let scr_height = screen_height();

    clear_background(DARK_BACKGROUND);

    // Define loading bar properties
    let bar_width = 300.0;
    let bar_height = 30.0;
    let bar_x = scr_width / 2.0 - bar_width / 2.0;
    let bar_y = scr_height / 2.0;

    // Measure the "Loading..." text width to center it above the loading bar
    let loading_text = "Loading...";
    let text_dimensions = measure_text(
        loading_text,
        Some(&assets.cyberpunk_font),
        CYBERPUNK_FONT_SIZE as u16,
        1.0
    );
    let text_x = (scr_width - text_dimensions.width) / 2.0; // Center horizontally
    let text_y = bar_y - 40.0; // Position 40 pixels above the loading bar

    // Draw "Loading..." text centered above the loading bar
    draw_text_ex(loading_text, text_x, text_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: CYBERPUNK_FONT_SIZE as u16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Draw neon loading bar
    let progress = (elapsed_time % 2.0) / 2.0;

    draw_rectangle(bar_x, bar_y, bar_width, bar_height, NEON_PURPLE);

    // Draw the progress
    draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, NEON_BLUE);

    // Add glow effect - reduced iterations for performance
    for i in 1..2 {
        let glow_alpha = 0.15 / (i as f32);
        draw_rectangle_lines(
            bar_x - (i as f32),
            bar_y - (i as f32),
            bar_width + 2.0 * (i as f32),
            bar_height + 2.0 * (i as f32),
            1.0,
            Color::new(NEON_PURPLE.r, NEON_PURPLE.g, NEON_PURPLE.b, glow_alpha)
        );
    }
}

/// Draw the score.
///
/// The score is drawn in the top right corner of the screen.
///
/// The `score` parameter is the current score.
///
/// The function draws a black rectangle with a white border and the score inside.
pub fn draw_score(score: i32, assets: &Assets) {
    let score_text = format!("Score: {}", score);

    // Neon glow effect behind the score
    draw_text_ex(&score_text, DRAW_SCORE_X + 4.0, DRAW_SCORE_Y + 4.0, TextParams {
        font: Some(&assets.cyberpunk_font), // Use the default font or your loaded cyberpunk font
        font_size: SCORE_FONT_SIZE as u16,
        color: Color::new(0.1, 0.1, 0.1, 0.8), // Soft shadow glow behind the score
        ..Default::default()
    });

    // Neon blue main score text
    draw_text_ex(&score_text, DRAW_SCORE_X, DRAW_SCORE_Y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: SCORE_FONT_SIZE as u16,
        color: NEON_BLUE, // Neon color for the score text
        ..Default::default()
    });
}

/// Draw the floating texts.
///
/// The `floating_texts` parameter is a vector of `FloatingText` structs containing the texts to draw.
///
/// The `elapsed` parameter is the elapsed time since the game started.
///
/// The function draws each text in the vector with a y offset based on the elapsed time.
pub fn draw_floating_texts(floating_texts: &mut Vec<FloatingText>, elapsed: f64, assets: &Assets) {
    // Use drain filter for more efficient cleanup
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
        let color = Color::new(1.0, 0.0, 0.0, alpha);

        draw_text_ex(&text.text, text.position.x, text.position.y - y_offset, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color,
            ..Default::default()
        });

        i += 1;
    }
}


