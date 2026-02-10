use macroquad::{
    color::{ WHITE, BLACK },
    input::{ is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton },
    prelude::Color,
    shapes::{ draw_line, draw_rectangle, draw_rectangle_lines, draw_circle },
    text::{ draw_text_ex, load_ttf_font, measure_text, TextParams },
    time::get_time,
    window::{ clear_background, screen_height, screen_width },
};
use crate::structs::{ 
    Assets, SongSelectionState, FloatingText, VisualizingState, PracticeMenuState, EndState 
};
use crate::constants::*;
use crate::config::{
    GameConfig, SettingsState, SettingsTab, KeyBindingType, get_available_keys, 
    BackgroundStyle, KeyBindings
};
use crate::analytics::{
    Analytics, AnalyticsState, AnalyticsView, Grade, HitStats
};
use std::fs;

/// Load all UI assets, such as textures and fonts.
pub async fn load_ui_assets() -> Assets {
    let cyberpunk_font = load_ttf_font("src/assets/fonts/teknaf.otf").await.unwrap();

    Assets {
        cyberpunk_font,
    }
}

/// Draw the main menu.
pub fn draw_menu(assets: &Assets) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let scr_width = screen_width();
    let scr_height = screen_height();

    let elapsed = get_time();

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
    let text_y = scr_height * 0.2;

    // Draw glowing title text
    draw_text_ex(title_text, text_x, text_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: font_size as u16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Button properties
    let button_width = BUTTON_WIDTH;
    let button_height = BUTTON_HEIGHT;
    let button_spacing = BUTTON_SPACING;

    // Calculate starting Y position for the buttons
    let start_y = scr_height * 0.4;

    // Create buttons with labels and positions
    let buttons = vec![
        ("Start Game", start_y),
        ("Practice", start_y + button_height + button_spacing),
        ("Analytics", start_y + 2.0 * (button_height + button_spacing)),
        ("Settings", start_y + 3.0 * (button_height + button_spacing)),
        ("Exit", start_y + 4.0 * (button_height + button_spacing)),
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

        // Change color when hovered with pulse effect
        let pulse = (elapsed.sin() as f32 * 0.2 + 0.8).max(0.6);
        let button_color = if is_hovered { 
            Color::new(NEON_GREEN.r * pulse, NEON_GREEN.g * pulse, NEON_GREEN.b * pulse, 1.0)
        } else { 
            NEON_BLUE 
        };

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

    selected_button
}

/// Draw the song selection menu.
pub fn draw_choose_audio(
    state: &mut SongSelectionState,
    songs: &[String],
    assets: &Assets
) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    let elapsed_time = get_time();

    // Draw the title at the top
    let title_text = "Select a Song";
    draw_text_ex(title_text, 20.0, screen_h * 0.1, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: CYBERPUNK_FONT_SIZE as u16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Draw practice mode indicator
    if state.practice_mode {
        let practice_text = format!("Practice Mode - {:.2}x Speed", state.playback_speed);
        draw_text_ex(&practice_text, 
            screen_w - 250.0, 
            screen_h * 0.1, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 18,
                color: NEON_YELLOW,
                ..Default::default()
            }
        );
    }

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

    let vertical_gap = 20.0;

    // Iterate through the songs and draw them as buttons
    for (i, song) in songs.iter().enumerate() {
        let button_x = screen_w * 0.05;
        let button_y =
            screen_h * 0.2 + (i as f32) * (SONG_ENTRY_HEIGHT + vertical_gap) - state.scroll_pos;

        if button_y > SONG_ENTRY_HEIGHT && button_y < screen_h - SONG_ENTRY_HEIGHT {
            let button_width = screen_w * 0.9;
            let button_height = SONG_ENTRY_HEIGHT;

            // Check if the button is hovered
            let mouse_pos = mouse_position();
            let is_hovered =
                mouse_pos.0 >= button_x &&
                mouse_pos.0 <= button_x + button_width &&
                mouse_pos.1 >= button_y &&
                mouse_pos.1 <= button_y + button_height;

            // Hover animation: Scale the button when hovered
            let scale_factor = if is_hovered { 1.05 } else { 1.0 };
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

            // Extract the song name
            let song_name = song
                .split('/')
                .last()
                .unwrap_or(song)
                .to_uppercase()
                .replace(".MP3", "")
                .replace(".mp3", "");

            // Measure text to center it within the scaled button
            let text_dimensions = measure_text(
                &song_name,
                Some(&assets.cyberpunk_font),
                CYBERPUNK_FONT_SIZE as u16,
                1.0
            );
            let text_x = scaled_button_x + (scaled_button_width - text_dimensions.width) / 2.0;
            let text_y = scaled_button_y + (scaled_button_height + text_dimensions.height) / 2.0;

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

    // Draw back button
    let back_text = "Press ESC to go back";
    draw_text_ex(back_text, 20.0, screen_h - 20.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: Color::new(1.0, 1.0, 1.0, 0.5),
        ..Default::default()
    });

    None
}

/// Load all songs from the assets directory.
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

/// Draw a loading bar.
pub fn draw_loading_bar(elapsed_time: f32, assets: &Assets, message: Option<&str>) {
    let scr_width = screen_width();
    let scr_height = screen_height();

    clear_background(DARK_BACKGROUND);

    // Define loading bar properties
    let bar_width = 300.0;
    let bar_height = 30.0;
    let bar_x = scr_width / 2.0 - bar_width / 2.0;
    let bar_y = scr_height / 2.0;

    // Loading message
    let loading_text = message.unwrap_or("Loading...");
    let text_dimensions = measure_text(
        loading_text,
        Some(&assets.cyberpunk_font),
        CYBERPUNK_FONT_SIZE as u16,
        1.0
    );
    let text_x = (scr_width - text_dimensions.width) / 2.0;
    let text_y = bar_y - 40.0;

    // Draw loading text
    draw_text_ex(loading_text, text_x, text_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: CYBERPUNK_FONT_SIZE as u16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Draw neon loading bar with animated progress
    let progress = ((elapsed_time % 2.0) / 2.0).min(1.0);

    // Background bar
    draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.2, 0.2, 0.3, 1.0));

    // Progress bar
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

    // Draw percentage text
    let percent_text = format!("{:.0}%", progress * 100.0);
    let percent_dimensions = measure_text(
        &percent_text,
        Some(&assets.cyberpunk_font),
        16,
        1.0
    );
    draw_text_ex(
        &percent_text,
        bar_x + (bar_width - percent_dimensions.width) / 2.0,
        bar_y + bar_height + 20.0,
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        }
    );
}

/// Draw the score with combo display
pub fn draw_score(score: i32, combo: u32, max_combo: u32, assets: &Assets) {
    // Draw combo if active
    if combo > 0 {
        let combo_text = format!("{}x", combo);
        let combo_size = if combo >= 100 {
            48
        } else if combo >= 50 {
            40
        } else if combo >= 25 {
            36
        } else {
            32
        };
        
        // Combo glow effect
        let pulse = (get_time() * 5.0).sin() as f32 * 0.3 + 0.7;
        let combo_color = if combo >= 100 {
            Color::new(1.0, 0.84 * pulse, 0.0, 1.0) // Gold
        } else if combo >= 50 {
            NEON_PINK
        } else if combo >= 25 {
            NEON_PURPLE
        } else {
            NEON_BLUE
        };
        
        let combo_y = DRAW_SCORE_Y + 50.0;
        
        draw_text_ex(&combo_text, 
            DRAW_SCORE_X + 2.0, 
            combo_y + 2.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: combo_size,
                color: Color::new(0.0, 0.0, 0.0, 0.5),
                ..Default::default()
            }
        );
        
        draw_text_ex(&combo_text, 
            DRAW_SCORE_X, 
            combo_y, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: combo_size,
                color: combo_color,
                ..Default::default()
            }
        );
    }

    // Draw score
    let score_text = format!("Score: {}", score);

    draw_text_ex(&score_text, DRAW_SCORE_X + 2.0, DRAW_SCORE_Y + 2.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: SCORE_FONT_SIZE as u16,
        color: Color::new(0.0, 0.0, 0.0, 0.5),
        ..Default::default()
    });

    draw_text_ex(&score_text, DRAW_SCORE_X, DRAW_SCORE_Y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: SCORE_FONT_SIZE as u16,
        color: NEON_BLUE,
        ..Default::default()
    });

    // Draw max combo
    let max_combo_text = format!("Max Combo: {}", max_combo);
    draw_text_ex(&max_combo_text, 
        DRAW_SCORE_X, 
        DRAW_SCORE_Y - 30.0, 
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: Color::new(1.0, 1.0, 1.0, 0.6),
            ..Default::default()
        }
    );
}

/// Draw the floating texts with improved visuals
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

/// Draw the settings menu
pub fn draw_settings(
    state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets
) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Draw title
    let title = "Settings";
    let title_dim = measure_text(title, Some(&assets.cyberpunk_font), 36, 1.0);
    draw_text_ex(title, (screen_w - title_dim.width) / 2.0, 60.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 36,
        color: NEON_PINK,
        ..Default::default()
    });

    // Draw tabs
    let tabs = SettingsTab::all();
    let tab_width = screen_w / tabs.len() as f32;
    
    for (i, (tab, name)) in tabs.iter().enumerate() {
        let tab_x = i as f32 * tab_width;
        let is_active = *tab == state.current_tab;
        
        let tab_color = if is_active { NEON_GREEN } else { NEON_BLUE };
        
        draw_rectangle(tab_x, 80.0, tab_width - 5.0, TAB_HEIGHT, tab_color);
        
        let tab_text_dim = measure_text(name, Some(&assets.cyberpunk_font), 18, 1.0);
        draw_text_ex(name, 
            tab_x + (tab_width - tab_text_dim.width) / 2.0,
            80.0 + (TAB_HEIGHT + tab_text_dim.height) / 2.0,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 18,
                color: if is_active { BLACK } else { WHITE },
                ..Default::default()
            }
        );

        // Check tab click
        let mouse_pos = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            if mouse_pos.0 >= tab_x && mouse_pos.0 <= tab_x + tab_width 
                && mouse_pos.1 >= 80.0 && mouse_pos.1 <= 80.0 + TAB_HEIGHT {
                state.current_tab = *tab;
            }
        }
    }

    // Draw content based on current tab
    let content_y = 140.0;
    match state.current_tab {
        SettingsTab::General => draw_general_settings(state, config, assets, content_y),
        SettingsTab::KeyBindings => draw_key_bindings_settings(state, config, assets, content_y),
        SettingsTab::Theme => draw_theme_settings(state, config, assets, content_y),
        SettingsTab::Audio => draw_audio_settings(state, config, assets, content_y),
        SettingsTab::Practice => draw_practice_settings(state, config, assets, content_y),
    }

    // Draw back button
    let back_text = "Press ESC to go back";
    draw_text_ex(back_text, 20.0, screen_h - 20.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: Color::new(1.0, 1.0, 1.0, 0.5),
        ..Default::default()
    });

    // Save button
    let save_x = screen_w - 150.0;
    let save_y = screen_h - 60.0;
    let mouse_pos = mouse_position();
    let save_hover = mouse_pos.0 >= save_x && mouse_pos.0 <= save_x + 120.0
        && mouse_pos.1 >= save_y && mouse_pos.1 <= save_y + 40.0;
    
    draw_rectangle(save_x, save_y, 120.0, 40.0, 
        if save_hover { NEON_GREEN } else { NEON_BLUE });
    draw_text_ex("Save", save_x + 35.0, save_y + 28.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    if is_mouse_button_pressed(MouseButton::Left) && save_hover {
        config.save();
        return Some("saved".to_string());
    }

    // Handle escape
    if is_key_pressed(KeyCode::Escape) {
        return Some("back".to_string());
    }

    None
}

fn draw_general_settings(
    _state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets,
    start_y: f32
) {
    let screen_w = screen_width();
    
    // Analytics toggle
    let y = start_y;
    draw_text_ex("Save Analytics:", 50.0, y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let checkbox_x = screen_w - 100.0;
    draw_checkbox(checkbox_x, y - 15.0, config.save_analytics, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= checkbox_x && mouse_pos.0 <= checkbox_x + 30.0
            && mouse_pos.1 >= y - 15.0 && mouse_pos.1 <= y + 15.0 {
            config.save_analytics = !config.save_analytics;
        }
    }

    // Reset button
    let reset_y = start_y + 60.0;
    let reset_text = "Reset to Defaults";
    let reset_dim = measure_text(reset_text, Some(&assets.cyberpunk_font), 20, 1.0);
    let reset_x = (screen_w - reset_dim.width) / 2.0;
    
    let mouse_pos = mouse_position();
    let reset_hover = mouse_pos.0 >= reset_x - 20.0 && mouse_pos.0 <= reset_x + reset_dim.width + 20.0
        && mouse_pos.1 >= reset_y - 10.0 && mouse_pos.1 <= reset_y + 30.0;
    
    draw_rectangle(reset_x - 20.0, reset_y - 10.0, reset_dim.width + 40.0, 40.0,
        if reset_hover { NEON_ORANGE } else { NEON_PINK });
    draw_text_ex(reset_text, reset_x, reset_y + 15.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    if is_mouse_button_pressed(MouseButton::Left) && reset_hover {
        config.reset_to_default();
    }
}

fn draw_key_bindings_settings(
    state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets,
    start_y: f32
) {
    let screen_w = screen_width();
    let bindings = KeyBindingType::all();
    
    // Handle waiting for key input
    if let Some(binding_type) = state.waiting_for_key {
        let prompt_text = format!("Press a key for: {}", binding_type.display_name());
        let prompt_dim = measure_text(&prompt_text, Some(&assets.cyberpunk_font), 24, 1.0);
        draw_text_ex(&prompt_text, 
            (screen_w - prompt_dim.width) / 2.0,
            start_y + 50.0,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 24,
                color: NEON_YELLOW,
                ..Default::default()
            }
        );

        // Check for key press
        for key in get_available_keys() {
            let keycode = super::config::string_to_keycode(key.0);
            if is_key_pressed(keycode) {
                let key_string = key.0.to_string();
                match binding_type {
                    KeyBindingType::PrimaryHit => config.key_bindings.primary_hit = key_string,
                    KeyBindingType::SecondaryHit => config.key_bindings.secondary_hit = key_string,
                    KeyBindingType::Pause => config.key_bindings.pause = key_string,
                    KeyBindingType::NavigateUp => config.key_bindings.navigate_up = key_string,
                    KeyBindingType::NavigateDown => config.key_bindings.navigate_down = key_string,
                    KeyBindingType::Select => config.key_bindings.select = key_string,
                }
                state.waiting_for_key = None;
                break;
            }
        }
        
        // Cancel on escape
        if is_key_pressed(KeyCode::Escape) {
            state.waiting_for_key = None;
        }
        
        return;
    }

    // Draw bindings
    for (i, binding_type) in bindings.iter().enumerate() {
        let y = start_y + i as f32 * 50.0;
        
        draw_text_ex(binding_type.display_name(), 50.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        });

        let key_string = match binding_type {
            KeyBindingType::PrimaryHit => &config.key_bindings.primary_hit,
            KeyBindingType::SecondaryHit => &config.key_bindings.secondary_hit,
            KeyBindingType::Pause => &config.key_bindings.pause,
            KeyBindingType::NavigateUp => &config.key_bindings.navigate_up,
            KeyBindingType::NavigateDown => &config.key_bindings.navigate_down,
            KeyBindingType::Select => &config.key_bindings.select,
        };

        let key_display = get_available_keys()
            .iter()
            .find(|(k, _)| *k == key_string.as_str())
            .map(|(_, d)| *d)
            .unwrap_or(key_string.as_str());

        let key_x = screen_w - 150.0;
        draw_rectangle(key_x - 10.0, y - 20.0, 120.0, 35.0, NEON_BLUE);
        draw_text_ex(key_display, key_x, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 18,
            color: WHITE,
            ..Default::default()
        });

        // Check click
        let mouse_pos = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            if mouse_pos.0 >= key_x - 10.0 && mouse_pos.0 <= key_x + 110.0
                && mouse_pos.1 >= y - 20.0 && mouse_pos.1 <= y + 15.0 {
                state.waiting_for_key = Some(*binding_type);
            }
        }
    }
}

fn draw_theme_settings(
    _state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets,
    start_y: f32
) {
    let screen_w = screen_width();

    // Circle size slider
    let y1 = start_y;
    draw_text_ex(&format!("Circle Size: {:.2}x", config.theme.circle_size), 
        50.0, y1, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        });

    let slider_x = screen_w - 250.0;
    draw_slider(slider_x, y1 - 10.0, 200.0, config.theme.circle_size, 0.5, 2.0);
    
    // Update on drag (simplified)
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= slider_x && mouse_pos.0 <= slider_x + 200.0
            && mouse_pos.1 >= y1 - 15.0 && mouse_pos.1 <= y1 + 5.0 {
            let ratio = (mouse_pos.0 - slider_x) / 200.0;
            config.theme.circle_size = 0.5 + ratio * 1.5;
        }
    }

    // Particles toggle
    let y2 = start_y + 50.0;
    draw_text_ex("Enable Particles:", 50.0, y2, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let checkbox_x = screen_w - 100.0;
    draw_checkbox(checkbox_x, y2 - 15.0, config.theme.particles_enabled, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= checkbox_x && mouse_pos.0 <= checkbox_x + 30.0
            && mouse_pos.1 >= y2 - 15.0 && mouse_pos.1 <= y2 + 15.0 {
            config.theme.particles_enabled = !config.theme.particles_enabled;
        }
    }

    // Screen shake toggle
    let y3 = start_y + 100.0;
    draw_text_ex("Screen Shake:", 50.0, y3, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    draw_checkbox(checkbox_x, y3 - 15.0, config.theme.screen_shake, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= checkbox_x && mouse_pos.0 <= checkbox_x + 30.0
            && mouse_pos.1 >= y3 - 15.0 && mouse_pos.1 <= y3 + 15.0 {
            config.theme.screen_shake = !config.theme.screen_shake;
        }
    }

    // Background style dropdown
    let y4 = start_y + 150.0;
    draw_text_ex("Background Style:", 50.0, y4, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let style_text = match config.theme.background_style {
        BackgroundStyle::Cyberpunk => "Cyberpunk",
        BackgroundStyle::Dark => "Dark",
        BackgroundStyle::Minimal => "Minimal",
        BackgroundStyle::Gradient => "Gradient",
    };

    draw_text_ex(style_text, screen_w - 200.0, y4, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: NEON_CYAN,
        ..Default::default()
    });
}

fn draw_audio_settings(
    _state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets,
    start_y: f32
) {
    let screen_w = screen_width();

    // Master volume
    let y1 = start_y;
    draw_text_ex(&format!("Master Volume: {:.0}%", config.audio.master_volume * 100.0), 
        50.0, y1, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        });

    let slider_x = screen_w - 250.0;
    draw_slider(slider_x, y1 - 10.0, 200.0, config.audio.master_volume, 0.0, 1.0);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= slider_x && mouse_pos.0 <= slider_x + 200.0
            && mouse_pos.1 >= y1 - 15.0 && mouse_pos.1 <= y1 + 5.0 {
            let ratio = (mouse_pos.0 - slider_x) / 200.0;
            config.audio.master_volume = ratio.clamp(0.0, 1.0);
        }
    }

    // Music volume
    let y2 = start_y + 50.0;
    draw_text_ex(&format!("Music Volume: {:.0}%", config.audio.music_volume * 100.0), 
        50.0, y2, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        });

    draw_slider(slider_x, y2 - 10.0, 200.0, config.audio.music_volume, 0.0, 1.0);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= slider_x && mouse_pos.0 <= slider_x + 200.0
            && mouse_pos.1 >= y2 - 15.0 && mouse_pos.1 <= y2 + 5.0 {
            let ratio = (mouse_pos.0 - slider_x) / 200.0;
            config.audio.music_volume = ratio.clamp(0.0, 1.0);
        }
    }

    // Effects volume
    let y3 = start_y + 100.0;
    draw_text_ex(&format!("Effects Volume: {:.0}%", config.audio.effects_volume * 100.0), 
        50.0, y3, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        });

    draw_slider(slider_x, y3 - 10.0, 200.0, config.audio.effects_volume, 0.0, 1.0);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= slider_x && mouse_pos.0 <= slider_x + 200.0
            && mouse_pos.1 >= y3 - 15.0 && mouse_pos.1 <= y3 + 5.0 {
            let ratio = (mouse_pos.0 - slider_x) / 200.0;
            config.audio.effects_volume = ratio.clamp(0.0, 1.0);
        }
    }
}

fn draw_practice_settings(
    _state: &mut SettingsState,
    config: &mut GameConfig,
    assets: &Assets,
    start_y: f32
) {
    let screen_w = screen_width();

    // Default playback speed
    let y1 = start_y;
    draw_text_ex(&format!("Default Speed: {:.2}x", config.practice.playback_speed), 
        50.0, y1, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: WHITE,
            ..Default::default()
        });

    let speeds = PracticeMenuState::speed_options();
    let speed_idx = speeds.iter().position(|(s, _)| *s == config.practice.playback_speed)
        .unwrap_or(3);
    
    let speed_text = speeds[speed_idx].1;
    draw_text_ex(speed_text, screen_w - 150.0, y1, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: NEON_CYAN,
        ..Default::default()
    });

    // Hit sounds toggle
    let y2 = start_y + 50.0;
    draw_text_ex("Hit Sounds:", 50.0, y2, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let checkbox_x = screen_w - 100.0;
    draw_checkbox(checkbox_x, y2 - 15.0, config.practice.hit_sounds, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= checkbox_x && mouse_pos.0 <= checkbox_x + 30.0
            && mouse_pos.1 >= y2 - 15.0 && mouse_pos.1 <= y2 + 15.0 {
            config.practice.hit_sounds = !config.practice.hit_sounds;
        }
    }
}

fn draw_checkbox(x: f32, y: f32, checked: bool, _assets: &Assets) {
    draw_rectangle(x, y, 30.0, 30.0, Color::new(0.2, 0.2, 0.3, 1.0));
    draw_rectangle_lines(x, y, 30.0, 30.0, 2.0, NEON_BLUE);
    
    if checked {
        draw_text_ex("✓", x + 6.0, y + 24.0, TextParams {
            font: None,
            font_size: 24,
            color: NEON_GREEN,
            ..Default::default()
        });
    }
}

fn draw_slider(x: f32, y: f32, width: f32, value: f32, min: f32, max: f32) {
    let ratio = (value - min) / (max - min);
    
    // Background
    draw_rectangle(x, y + 5.0, width, 10.0, Color::new(0.2, 0.2, 0.3, 1.0));
    
    // Fill
    draw_rectangle(x, y + 5.0, width * ratio, 10.0, NEON_BLUE);
    
    // Handle
    draw_circle(x + width * ratio, y + 10.0, 8.0, NEON_GREEN);
}

/// Draw the analytics screen
pub fn draw_analytics(
    state: &mut AnalyticsState,
    analytics: &Analytics,
    assets: &Assets
) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Draw title
    let title = "Analytics";
    let title_dim = measure_text(title, Some(&assets.cyberpunk_font), 36, 1.0);
    draw_text_ex(title, (screen_w - title_dim.width) / 2.0, 60.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 36,
        color: NEON_PINK,
        ..Default::default()
    });

    // Draw tabs
    let views = AnalyticsView::all();
    let tab_width = screen_w / views.len() as f32;
    
    for (i, (view, name)) in views.iter().enumerate() {
        let tab_x = i as f32 * tab_width;
        let is_active = *view == state.current_view;
        
        let tab_color = if is_active { NEON_GREEN } else { NEON_BLUE };
        
        draw_rectangle(tab_x, 80.0, tab_width - 5.0, TAB_HEIGHT, tab_color);
        
        let tab_text_dim = measure_text(name, Some(&assets.cyberpunk_font), 16, 1.0);
        draw_text_ex(name, 
            tab_x + (tab_width - tab_text_dim.width) / 2.0,
            80.0 + (TAB_HEIGHT + tab_text_dim.height) / 2.0,
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 16,
                color: if is_active { BLACK } else { WHITE },
                ..Default::default()
            }
        );

        // Check tab click
        let mouse_pos = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            if mouse_pos.0 >= tab_x && mouse_pos.0 <= tab_x + tab_width 
                && mouse_pos.1 >= 80.0 && mouse_pos.1 <= 80.0 + TAB_HEIGHT {
                state.current_view = *view;
            }
        }
    }

    // Draw content
    let content_y = 140.0;
    match state.current_view {
        AnalyticsView::Overview => draw_analytics_overview(analytics, assets, content_y),
        AnalyticsView::Songs => draw_analytics_songs(analytics, assets, content_y),
        AnalyticsView::Sessions => draw_analytics_sessions(analytics, assets, content_y, state),
        AnalyticsView::Achievements => draw_analytics_achievements(analytics, assets, content_y),
        AnalyticsView::Trends => draw_analytics_trends(analytics, assets, content_y),
    }

    // Draw back button
    let back_text = "Press ESC to go back";
    draw_text_ex(back_text, 20.0, screen_h - 20.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: Color::new(1.0, 1.0, 1.0, 0.5),
        ..Default::default()
    });

    // Handle escape
    if is_key_pressed(KeyCode::Escape) {
        return Some("back".to_string());
    }

    None
}

fn draw_analytics_overview(analytics: &Analytics, assets: &Assets, start_y: f32) {
    let screen_w = screen_width();
    let stats = analytics.get_overall_stats();

    let stats_items = vec![
        ("Total Games Played", stats.total_games.to_string()),
        ("Total Play Time", stats.format_play_time()),
        ("Overall Accuracy", format!("{:.1}%", stats.overall_accuracy)),
        ("Full Combos", stats.total_full_combos.to_string()),
        ("Best Grade", stats.best_overall_grade.map(|g| g.as_str().to_string()).unwrap_or_else(|| "-".to_string())),
    ];

    for (i, (label, value)) in stats_items.iter().enumerate() {
        let y = start_y + i as f32 * 50.0;
        
        draw_text_ex(label, 50.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: Color::new(0.8, 0.8, 0.8, 1.0),
            ..Default::default()
        });

        let value_color = if label.contains("Accuracy") {
            if stats.overall_accuracy >= 90.0 { NEON_GREEN }
            else if stats.overall_accuracy >= 70.0 { NEON_YELLOW }
            else { NEON_ORANGE }
        } else if label.contains("Grade") {
            match value.as_str() {
                "SS" => GRADE_SS_COLOR,
                "S" => GRADE_S_COLOR,
                "A" => GRADE_A_COLOR,
                "B" => GRADE_B_COLOR,
                "C" => GRADE_C_COLOR,
                _ => WHITE,
            }
        } else {
            NEON_CYAN
        };

        let value_x = screen_w - 200.0;
        draw_text_ex(value, value_x, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 24,
            color: value_color,
            ..Default::default()
        });
    }

    // Hit stats breakdown
    let hit_y = start_y + 300.0;
    draw_text_ex("Hit Breakdown:", 50.0, hit_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 22,
        color: NEON_PINK,
        ..Default::default()
    });

    let hit_stats = vec![
        ("Perfect (300)", analytics.total_hits.perfect, NEON_GREEN),
        ("Good (100)", analytics.total_hits.good, NEON_BLUE),
        ("Okay (50)", analytics.total_hits.okay, NEON_YELLOW),
        ("Misses", analytics.total_hits.misses, NEON_ORANGE),
    ];

    for (i, (label, count, color)) in hit_stats.iter().enumerate() {
        let y = hit_y + 40.0 + i as f32 * 35.0;
        draw_text_ex(&format!("{}: {}", label, count), 70.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 18,
            color: *color,
            ..Default::default()
        });
    }
}

fn draw_analytics_songs(analytics: &Analytics, assets: &Assets, start_y: f32) {
    let screen_w = screen_width();
    let most_played = analytics.get_most_played_songs(10);

    if most_played.is_empty() {
        draw_text_ex("No songs played yet!", 
            screen_w / 2.0 - 100.0, 
            start_y + 100.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 20,
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                ..Default::default()
            }
        );
        return;
    }

    // Headers
    draw_text_ex("Song", 50.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 18,
        color: NEON_PINK,
        ..Default::default()
    });
    draw_text_ex("Plays", screen_w - 300.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 18,
        color: NEON_PINK,
        ..Default::default()
    });
    draw_text_ex("Best Score", screen_w - 180.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 18,
        color: NEON_PINK,
        ..Default::default()
    });

    // Song list
    for (i, (song_name, stats)) in most_played.iter().enumerate() {
        let y = start_y + 30.0 + i as f32 * 35.0;
        
        let display_name = if song_name.len() > 30 {
            format!("{}...", &song_name[..27])
        } else {
            song_name.clone()
        };

        draw_text_ex(&display_name, 50.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        });

        draw_text_ex(&stats.play_count.to_string(), screen_w - 280.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: NEON_CYAN,
            ..Default::default()
        });

        draw_text_ex(&stats.best_score.to_string(), screen_w - 160.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: NEON_GREEN,
            ..Default::default()
        });
    }
}

fn draw_analytics_sessions(
    analytics: &Analytics, 
    assets: &Assets, 
    start_y: f32,
    state: &mut AnalyticsState
) {
    let screen_w = screen_width();
    let recent_sessions: Vec<_> = analytics.recent_sessions.iter().rev().take(10).collect();

    if recent_sessions.is_empty() {
        draw_text_ex("No sessions recorded yet!", 
            screen_w / 2.0 - 120.0, 
            start_y + 100.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 20,
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                ..Default::default()
            }
        );
        return;
    }

    // Headers
    draw_text_ex("Song", 30.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: NEON_PINK,
        ..Default::default()
    });
    draw_text_ex("Score", screen_w - 280.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: NEON_PINK,
        ..Default::default()
    });
    draw_text_ex("Acc", screen_w - 190.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: NEON_PINK,
        ..Default::default()
    });
    draw_text_ex("Grade", screen_w - 130.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: NEON_PINK,
        ..Default::default()
    });

    // Session list
    for (i, session) in recent_sessions.iter().enumerate() {
        let y = start_y + 25.0 + i as f32 * 30.0;
        
        let song_name = session.song_name.split('/').last()
            .unwrap_or(&session.song_name)
            .replace(".mp3", "")
            .replace(".ogg", "");
        
        let display_name = if song_name.len() > 25 {
            format!("{}...", &song_name[..22])
        } else {
            song_name
        };

        draw_text_ex(&display_name, 30.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 14,
            color: WHITE,
            ..Default::default()
        });

        draw_text_ex(&session.score.to_string(), screen_w - 280.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 14,
            color: NEON_CYAN,
            ..Default::default()
        });

        let acc_color = if session.accuracy >= 90.0 { NEON_GREEN }
            else if session.accuracy >= 75.0 { NEON_YELLOW }
            else { NEON_ORANGE };

        draw_text_ex(&format!("{:.1}%", session.accuracy), 
            screen_w - 190.0, y, TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 14,
                color: acc_color,
                ..Default::default()
            });

        let grade_color = match session.grade {
            Grade::SS => GRADE_SS_COLOR,
            Grade::S => GRADE_S_COLOR,
            Grade::A => GRADE_A_COLOR,
            Grade::B => GRADE_B_COLOR,
            Grade::C => GRADE_C_COLOR,
            Grade::D => GRADE_D_COLOR,
            Grade::F => GRADE_F_COLOR,
        };

        draw_text_ex(session.grade.as_str(), screen_w - 130.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: grade_color,
            ..Default::default()
        });

        // FC indicator
        if session.full_combo {
            draw_text_ex("FC", screen_w - 90.0, y, TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 12,
                color: NEON_GREEN,
                ..Default::default()
            });
        }
    }
}

fn draw_analytics_achievements(analytics: &Analytics, assets: &Assets, start_y: f32) {
    let screen_w = screen_width();

    if analytics.achievements.is_empty() {
        draw_text_ex("No achievements unlocked yet!", 
            screen_w / 2.0 - 140.0, 
            start_y + 100.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 20,
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                ..Default::default()
            }
        );
        return;
    }

    draw_text_ex(&format!("Achievements Unlocked: {}", analytics.achievements.len()), 
        50.0, start_y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: NEON_YELLOW,
            ..Default::default()
        });

    // Achievement list
    for (i, achievement) in analytics.achievements.iter().enumerate() {
        let y = start_y + 40.0 + i as f32 * 45.0;
        
        draw_text_ex(&achievement.name, 50.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 18,
            color: NEON_GREEN,
            ..Default::default()
        });

        draw_text_ex(&achievement.description, 70.0, y + 20.0, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 14,
            color: Color::new(0.7, 0.7, 0.7, 1.0),
            ..Default::default()
        });
    }
}

fn draw_analytics_trends(analytics: &Analytics, assets: &Assets, start_y: f32) {
    let screen_w = screen_width();

    let trend = analytics.get_accuracy_trend();
    
    if trend.len() < 2 {
        draw_text_ex("Not enough data to show trends!", 
            screen_w / 2.0 - 140.0, 
            start_y + 100.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 20,
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                ..Default::default()
            }
        );
        return;
    }

    draw_text_ex("Accuracy Trend (Last 10 Games)", 50.0, start_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: NEON_PINK,
        ..Default::default()
    });

    // Simple trend visualization
    let chart_y = start_y + 50.0;
    let chart_height = 150.0;
    let chart_width = screen_w - 100.0;
    
    // Draw chart background
    draw_rectangle(50.0, chart_y, chart_width, chart_height, Color::new(0.1, 0.1, 0.15, 1.0));
    draw_rectangle_lines(50.0, chart_y, chart_width, chart_height, 2.0, NEON_BLUE);

    // Draw trend line
    let x_step = chart_width / (trend.len() - 1).max(1) as f32;
    
    for (i, accuracy) in trend.iter().enumerate() {
        let x = 50.0 + i as f32 * x_step;
        let y = chart_y + chart_height - (accuracy / 100.0) * chart_height;
        
        let point_color = if *accuracy >= 90.0 { NEON_GREEN }
            else if *accuracy >= 75.0 { NEON_YELLOW }
            else { NEON_ORANGE };
        
        draw_circle(x, y, 5.0, point_color);

        // Draw accuracy value
        if i % 2 == 0 {
            draw_text_ex(&format!("{:.0}", accuracy), x - 10.0, y - 10.0, TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 12,
                color: point_color,
                ..Default::default()
            });
        }
    }

    // Average accuracy
    let avg: f32 = trend.iter().sum::<f32>() / trend.len() as f32;
    draw_text_ex(&format!("Recent Average: {:.1}%", avg), 
        50.0, chart_y + chart_height + 40.0, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 18,
            color: NEON_CYAN,
            ..Default::default()
        });
}

/// Draw the practice menu
pub fn draw_practice_menu(
    state: &mut PracticeMenuState,
    songs: &[String],
    assets: &Assets
) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Draw title
    let title = "Practice Mode";
    let title_dim = measure_text(title, Some(&assets.cyberpunk_font), 36, 1.0);
    draw_text_ex(title, (screen_w - title_dim.width) / 2.0, 60.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 36,
        color: NEON_YELLOW,
        ..Default::default()
    });

    // Draw options
    let option_y_start = 120.0;
    let option_spacing = 60.0;

    // Playback speed
    let speed_y = option_y_start;
    draw_text_ex("Playback Speed:", 50.0, speed_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let speeds = PracticeMenuState::speed_options();
    let speed_idx = speeds.iter().position(|(s, _)| *s == state.playback_speed)
        .unwrap_or(3);
    let speed_text = speeds[speed_idx].1;

    let speed_button_x = screen_w - 200.0;
    let speed_hover = draw_option_button(speed_button_x, speed_y - 25.0, 100.0, speed_text, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) && speed_hover {
        let next_idx = (speed_idx + 1) % speeds.len();
        state.playback_speed = speeds[next_idx].0;
    }

    // No-fail mode
    let nofail_y = option_y_start + option_spacing;
    draw_text_ex("No-Fail Mode:", 50.0, nofail_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let nofail_checkbox_x = screen_w - 100.0;
    draw_checkbox(nofail_checkbox_x, nofail_y - 15.0, state.no_fail, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= nofail_checkbox_x && mouse_pos.0 <= nofail_checkbox_x + 30.0
            && mouse_pos.1 >= nofail_y - 15.0 && mouse_pos.1 <= nofail_y + 15.0 {
            state.no_fail = !state.no_fail;
        }
    }

    // Autoplay
    let autoplay_y = option_y_start + option_spacing * 2.0;
    draw_text_ex("Autoplay:", 50.0, autoplay_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let autoplay_checkbox_x = screen_w - 100.0;
    draw_checkbox(autoplay_checkbox_x, autoplay_y - 15.0, state.autoplay, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= autoplay_checkbox_x && mouse_pos.0 <= autoplay_checkbox_x + 30.0
            && mouse_pos.1 >= autoplay_y - 15.0 && mouse_pos.1 <= autoplay_y + 15.0 {
            state.autoplay = !state.autoplay;
        }
    }

    // Hit sounds
    let hitsound_y = option_y_start + option_spacing * 3.0;
    draw_text_ex("Hit Sounds:", 50.0, hitsound_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let hitsound_checkbox_x = screen_w - 100.0;
    draw_checkbox(hitsound_checkbox_x, hitsound_y - 15.0, state.hit_sounds, assets);
    
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        if mouse_pos.0 >= hitsound_checkbox_x && mouse_pos.0 <= hitsound_checkbox_x + 30.0
            && mouse_pos.1 >= hitsound_y - 15.0 && mouse_pos.1 <= hitsound_y + 15.0 {
            state.hit_sounds = !state.hit_sounds;
        }
    }

    // Song selection header
    draw_text_ex("Select a Song:", 50.0, 360.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 22,
        color: NEON_PINK,
        ..Default::default()
    });

    // Draw song list (simplified)
    let song_start_y = 390.0;
    let song_height = 35.0;
    
    for (i, song) in songs.iter().take(5).enumerate() {
        let y = song_start_y + i as f32 * song_height;
        
        let song_name = song.split('/').last()
            .unwrap_or(song)
            .replace(".mp3", "")
            .replace(".ogg", "")
            .to_uppercase();

        let is_selected = state.selected_song.as_ref() == Some(song);
        let button_color = if is_selected { NEON_GREEN } else { NEON_BLUE };
        
        let button_width = screen_w - 100.0;
        let mouse_pos = mouse_position();
        let is_hovered = mouse_pos.0 >= 50.0 && mouse_pos.0 <= 50.0 + button_width
            && mouse_pos.1 >= y - 20.0 && mouse_pos.1 <= y + 10.0;

        draw_rectangle(50.0, y - 20.0, button_width, 30.0, 
            if is_hovered { NEON_PURPLE } else { button_color });

        draw_text_ex(&song_name, 60.0, y, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        });

        if is_mouse_button_pressed(MouseButton::Left) && is_hovered {
            state.selected_song = Some(song.clone());
        }
    }

    // Start button
    let start_y = screen_h - 100.0;
    let start_text = "Start Practice";
    let start_dim = measure_text(start_text, Some(&assets.cyberpunk_font), 24, 1.0);
    let start_x = (screen_w - start_dim.width) / 2.0;
    
    let start_enabled = state.selected_song.is_some();
    let start_hover = if start_enabled {
        let mouse_pos = mouse_position();
        mouse_pos.0 >= start_x - 30.0 && mouse_pos.0 <= start_x + start_dim.width + 30.0
            && mouse_pos.1 >= start_y - 15.0 && mouse_pos.1 <= start_y + 25.0
    } else {
        false
    };

    let start_color = if !start_enabled {
        Color::new(0.3, 0.3, 0.3, 1.0)
    } else if start_hover {
        NEON_GREEN
    } else {
        NEON_BLUE
    };

    draw_rectangle(start_x - 30.0, start_y - 15.0, start_dim.width + 60.0, 40.0, start_color);
    draw_text_ex(start_text, start_x, start_y + 10.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 24,
        color: if start_enabled { WHITE } else { Color::new(0.5, 0.5, 0.5, 1.0) },
        ..Default::default()
    });

    if start_enabled && start_hover && is_mouse_button_pressed(MouseButton::Left) {
        return Some("start".to_string());
    }

    // Back button
    let back_text = "Press ESC to go back";
    draw_text_ex(back_text, 20.0, screen_h - 20.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 16,
        color: Color::new(1.0, 1.0, 1.0, 0.5),
        ..Default::default()
    });

    if is_key_pressed(KeyCode::Escape) {
        return Some("back".to_string());
    }

    None
}

fn draw_option_button(x: f32, y: f32, width: f32, text: &str, assets: &Assets) -> bool {
    let text_dim = measure_text(text, Some(&assets.cyberpunk_font), 18, 1.0);
    let height = 30.0;
    
    let mouse_pos = mouse_position();
    let is_hovered = mouse_pos.0 >= x && mouse_pos.0 <= x + width
        && mouse_pos.1 >= y && mouse_pos.1 <= y + height;

    draw_rectangle(x, y, width, height, 
        if is_hovered { NEON_PURPLE } else { NEON_BLUE });

    draw_text_ex(text, x + (width - text_dim.width) / 2.0, y + height - 7.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 18,
        color: WHITE,
        ..Default::default()
    });

    is_hovered
}

/// Draw the end screen with results
pub fn draw_end_screen(state: &EndState, assets: &Assets) -> Option<String> {
    clear_background(DARK_BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Title
    let title = "Results";
    let title_dim = measure_text(title, Some(&assets.cyberpunk_font), 48, 1.0);
    draw_text_ex(title, (screen_w - title_dim.width) / 2.0, 80.0, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 48,
        color: NEON_PINK,
        ..Default::default()
    });

    // Grade display (large)
    let grade_color = match state.grade {
        Grade::SS => GRADE_SS_COLOR,
        Grade::S => GRADE_S_COLOR,
        Grade::A => GRADE_A_COLOR,
        Grade::B => GRADE_B_COLOR,
        Grade::C => GRADE_C_COLOR,
        Grade::D => GRADE_D_COLOR,
        Grade::F => GRADE_F_COLOR,
    };

    let grade_text = state.grade.as_str();
    let grade_dim = measure_text(grade_text, Some(&assets.cyberpunk_font), 120, 1.0);
    
    // Grade glow effect
    let pulse = (get_time() * 3.0).sin() as f32 * 0.2 + 0.8;
    let glow_color = Color::new(
        grade_color.r * pulse,
        grade_color.g * pulse,
        grade_color.b * pulse,
        1.0
    );

    draw_text_ex(grade_text, 
        (screen_w - grade_dim.width) / 2.0, 
        200.0, 
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 120,
            color: glow_color,
            ..Default::default()
        }
    );

    // Full combo indicator
    if state.full_combo {
        let fc_text = "FULL COMBO!";
        let fc_dim = measure_text(fc_text, Some(&assets.cyberpunk_font), 28, 1.0);
        draw_text_ex(fc_text, 
            (screen_w - fc_dim.width) / 2.0, 
            240.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 28,
                color: NEON_GREEN,
                ..Default::default()
            }
        );
    }

    // New best indicator
    if state.new_best {
        let new_best_text = "NEW BEST!";
        let new_best_dim = measure_text(new_best_text, Some(&assets.cyberpunk_font), 24, 1.0);
        draw_text_ex(new_best_text, 
            (screen_w - new_best_dim.width) / 2.0, 
            270.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 24,
                color: NEON_YELLOW,
                ..Default::default()
            }
        );
    }

    // Stats
    let stats_x = screen_w / 2.0 - 100.0;
    let stats_y = 320.0;
    let stats_spacing = 35.0;

    draw_text_ex(&format!("Score: {}", state.score), stats_x, stats_y, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 24,
        color: NEON_CYAN,
        ..Default::default()
    });

    draw_text_ex(&format!("Max Combo: {}", state.max_combo), stats_x, stats_y + stats_spacing, TextParams {
        font: Some(&assets.cyberpunk_font),
        font_size: 20,
        color: WHITE,
        ..Default::default()
    });

    let acc_color = if state.accuracy >= 90.0 { NEON_GREEN }
        else if state.accuracy >= 75.0 { NEON_YELLOW }
        else { NEON_ORANGE };

    draw_text_ex(&format!("Accuracy: {:.1}%", state.accuracy), 
        stats_x, stats_y + stats_spacing * 2.0, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: acc_color,
            ..Default::default()
        });

    // Hit breakdown
    draw_text_ex(&format!("Perfect: {}", state.hits.perfect), 
        stats_x, stats_y + stats_spacing * 3.5, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: NEON_GREEN,
            ..Default::default()
        });

    draw_text_ex(&format!("Good: {}", state.hits.good), 
        stats_x + 100.0, stats_y + stats_spacing * 3.5, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: NEON_BLUE,
            ..Default::default()
        });

    draw_text_ex(&format!("Okay: {}", state.hits.okay), 
        stats_x, stats_y + stats_spacing * 4.2, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: NEON_YELLOW,
            ..Default::default()
        });

    draw_text_ex(&format!("Miss: {}", state.hits.misses), 
        stats_x + 100.0, stats_y + stats_spacing * 4.2, TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 16,
            color: NEON_ORANGE,
            ..Default::default()
        });

    // Practice mode indicator
    if state.practice_mode {
        let practice_text = format!("Practice Mode - {:.2}x Speed", state.playback_speed);
        let practice_dim = measure_text(&practice_text, Some(&assets.cyberpunk_font), 16, 1.0);
        draw_text_ex(&practice_text, 
            (screen_w - practice_dim.width) / 2.0, 
            screen_h - 120.0, 
            TextParams {
                font: Some(&assets.cyberpunk_font),
                font_size: 16,
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                ..Default::default()
            }
        );
    }

    // Continue prompt
    let prompt = "Press ENTER or Click to continue";
    let prompt_dim = measure_text(prompt, Some(&assets.cyberpunk_font), 20, 1.0);
    let prompt_pulse = (get_time() * 2.0).sin() as f32 * 0.3 + 0.7;
    
    draw_text_ex(prompt, 
        (screen_w - prompt_dim.width) / 2.0, 
        screen_h - 60.0, 
        TextParams {
            font: Some(&assets.cyberpunk_font),
            font_size: 20,
            color: Color::new(1.0, 1.0, 1.0, prompt_pulse),
            ..Default::default()
        }
    );

    // Check for continue
    if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
        return Some("continue".to_string());
    }

    None
}
