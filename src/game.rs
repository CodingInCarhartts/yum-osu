use crate::constants::*;
use crate::gamemode::{GameSettings, Modifier};
use crate::structs::{FloatingText, GameCircle, VisualizingState};
use bevy::prelude::*;
use rand::Rng;

/// Component marker for game circles
#[derive(Component)]
pub struct CircleComponent {
    pub circle_index: usize,
}

/// Initialize circles for a game with animations
pub fn initialize_circles(
    beats: &[f64],
    rng: &mut impl Rng,
    spawn_radius: f32,
    center: Vec2,
    shrink_time: f64,
    delay: f64,
    config: &crate::config::GameConfig,
) -> Vec<GameCircle> {
    let game_settings = &config.game_settings;
    let mut circles = Vec::with_capacity(beats.len());

    // Apply difficulty multipliers
    let circle_size_mult = game_settings.difficulty.circle_size_multiplier();
    let shrink_time_mult = game_settings.difficulty.shrink_time_multiplier();

    for &beat_time in beats {
        let (angle, distance) = if game_settings.randomize_positions() {
            (
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..spawn_radius),
            )
        } else {
            (
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..spawn_radius),
            )
        };

        let position = Vec2::new(
            center.x + distance * angle.cos(),
            center.y + distance * angle.sin(),
        );

        let adjusted_shrink_time = shrink_time * shrink_time_mult;
        let max_radius = CIRCLE_MAX_RADIUS * circle_size_mult * config.theme.circle_size;

        circles.push(GameCircle {
            position,
            spawn_time: beat_time - adjusted_shrink_time + delay,
            hit_time: beat_time + delay,
            max_radius,
            hit: false,
            missed: false,
        });
    }

    circles
}

/// Calculate the spawn radius based on the screen size
pub fn calculate_spawn_radius(width: f32, height: f32) -> f32 {
    width.min(height) / 2.0 - 100.0
}

/// Calculate the shrinking radius with animation
pub fn circle_radius(circle: &GameCircle, elapsed: f64, shrink_time: f64) -> Option<f32> {
    let time_since_spawn = elapsed - circle.spawn_time;
    if (0.0..=shrink_time).contains(&time_since_spawn) {
        Some(circle.max_radius * (1.0 - ((time_since_spawn / shrink_time) as f32)))
    } else {
        None
    }
}

/// Calculate score from timing difference, applying modifiers and game settings
pub fn calculate_score_from_timing(time_difference: f64, game_settings: &GameSettings) -> i32 {
    let base_score = if time_difference < 0.08 {
        300
    } else if time_difference < 0.2 {
        100
    } else if time_difference < 0.35 {
        50
    } else {
        0
    };

    // Apply Perfect Only modifier
    if game_settings.perfect_only() && base_score < 300 {
        return 0;
    }

    // Apply score multiplier
    let multiplier = game_settings.score_multiplier();
    (base_score as f32 * multiplier) as i32
}

/// Legacy version for backward compatibility
pub fn calculate_score_from_timing_legacy(time_difference: f64) -> i32 {
    calculate_score_from_timing(time_difference, &GameSettings::default())
}

/// Handle missed circles and animate a "Miss" text
/// Returns true if the game should end (e.g., survival mode with no lives)
pub fn handle_missed_circles(
    circles: &mut Vec<GameCircle>,
    elapsed: f64,
    vis_state: &mut VisualizingState,
    shrink_time: f64,
) -> bool {
    let mut should_end_game = false;

    for circle in circles.iter_mut().filter(|c| !c.hit && !c.missed) {
        let time_since_spawn = elapsed - circle.spawn_time;

        if time_since_spawn > shrink_time {
            circle.missed = true;

            // Handle survival mode
            if let Some(ref mut lives) = vis_state.lives {
                *lives = lives.saturating_sub(1);
                if *lives == 0 {
                    should_end_game = true;
                }

                vis_state.floating_texts.push(FloatingText {
                    text: format!("Lives: {}", *lives),
                    position: circle.position,
                    spawn_time: elapsed,
                    duration: 1.5,
                    color: (1.0, 0.5, 0.0),
                });
            }

            // Only record miss if not in no-fail mode
            if !vis_state.no_fail && !vis_state.game_settings.has_modifier(Modifier::NoFail) {
                vis_state.record_miss();
            }

            vis_state.floating_texts.push(FloatingText {
                text: "Miss".to_string(),
                position: circle.position,
                spawn_time: elapsed,
                duration: 1.0,
                color: (1.0, 0.0, 0.0),
            });
        }
    }

    should_end_game
}

/// Score calculation based on the hit time and elapsed time (legacy)
pub fn calculate_score(hit_time: f64, current_time: f64) -> i32 {
    let time_difference = (current_time - hit_time).abs();
    calculate_score_from_timing(time_difference)
}

/// Draw circles in Bevy
pub fn draw_circles_bevy(
    commands: &mut Commands,
    circles: &[GameCircle],
    elapsed: f64,
    shrink_time: f64,
    game_settings: &GameSettings,
) {
    // Pre-compute pulse intensity once
    let pulse_intensity = 0.5 + (elapsed.sin() as f32) * 0.5;

    let show_approach = game_settings.show_approach_circles();

    for circle in circles {
        let time_since_spawn = elapsed - circle.spawn_time;

        if (0.0..=shrink_time).contains(&time_since_spawn) && !circle.hit {
            // Shrink circle with a smooth scaling effect
            let scale = 1.0 - (time_since_spawn / shrink_time) as f32;
            let radius = circle.max_radius * scale;

            // Cull circles that are too small to see
            if radius < 1.0 {
                continue;
            }

            // Pre-compute alpha
            let alpha = 0.6 - scale * 0.5;

            // Draw outline circle (pulsing effect)
            commands.spawn((
                Sprite {
                    color: Color::srgba(
                        OUTLINE_COLOR.to_linear().red,
                        OUTLINE_COLOR.to_linear().green,
                        OUTLINE_COLOR.to_linear().blue,
                        pulse_intensity,
                    ),
                    custom_size: Some(Vec2::new(
                        (radius + OUTLINE_THICKNESS) * 2.0,
                        (radius + OUTLINE_THICKNESS) * 2.0,
                    )),
                    ..default()
                },
                Transform::from_xyz(circle.position.x, circle.position.y, 0.3),
                crate::ui::UiElement,
            ));

            // Draw main circle
            let color = Color::srgba(0.0, 0.75, 1.0, alpha);
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                    ..default()
                },
                Transform::from_xyz(circle.position.x, circle.position.y, 0.2),
                crate::ui::UiElement,
            ));

            // Draw approach circle (outline) only if not hidden
            if show_approach {
                let approach_alpha = 0.3 + pulse_intensity * 0.3;
                commands.spawn((
                    Sprite {
                        color: Color::srgba(
                            OUTLINE_COLOR.to_linear().red,
                            OUTLINE_COLOR.to_linear().green,
                            OUTLINE_COLOR.to_linear().blue,
                            approach_alpha,
                        ),
                        custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(circle.position.x, circle.position.y, 0.1),
                    crate::ui::UiElement,
                ));
            }
        }
    }
}

            // Pre-compute alpha
            let alpha = 0.6 - scale * 0.5;

            // Draw outline circle (pulsing effect)
            commands.spawn((
                Sprite {
                    color: Color::srgba(
                        OUTLINE_COLOR.to_linear().red,
                        OUTLINE_COLOR.to_linear().green,
                        OUTLINE_COLOR.to_linear().blue,
                        pulse_intensity,
                    ),
                    custom_size: Some(Vec2::new(
                        (radius + OUTLINE_THICKNESS) * 2.0,
                        (radius + OUTLINE_THICKNESS) * 2.0,
                    )),
                    ..default()
                },
                Transform::from_xyz(circle.position.x, circle.position.y, 0.3),
                crate::ui::UiElement,
            ));

            // Draw main circle
            let color = Color::srgba(0.0, 0.75, 1.0, alpha);
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                    ..default()
                },
                Transform::from_xyz(circle.position.x, circle.position.y, 0.2),
                crate::ui::UiElement,
            ));

            // Draw approach circle (outline)
            let approach_alpha = 0.3 + pulse_intensity * 0.3;
            commands.spawn((
                Sprite {
                    color: Color::srgba(
                        OUTLINE_COLOR.to_linear().red,
                        OUTLINE_COLOR.to_linear().green,
                        OUTLINE_COLOR.to_linear().blue,
                        approach_alpha,
                    ),
                    custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                    ..default()
                },
                Transform::from_xyz(circle.position.x, circle.position.y, 0.1),
                crate::ui::UiElement,
            ));
        }
    }
}
