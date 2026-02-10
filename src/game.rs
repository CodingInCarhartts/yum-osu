use crate::constants::*;
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
    _config: &crate::config::GameConfig,
) -> Vec<GameCircle> {
    let mut circles = Vec::with_capacity(beats.len());

    for &beat_time in beats {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(0.0..spawn_radius);

        let position = Vec2::new(
            center.x + distance * angle.cos(),
            center.y + distance * angle.sin(),
        );

        circles.push(GameCircle {
            position,
            spawn_time: beat_time - shrink_time + delay,
            hit_time: beat_time + delay,
            max_radius: CIRCLE_MAX_RADIUS,
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

/// Calculate score from timing difference
pub fn calculate_score_from_timing(time_difference: f64) -> i32 {
    if time_difference < 0.08 {
        300
    } else if time_difference < 0.2 {
        100
    } else if time_difference < 0.35 {
        50
    } else {
        0
    }
}

/// Handle missed circles and animate a "Miss" text
pub fn handle_missed_circles(
    circles: &mut Vec<GameCircle>,
    elapsed: f64,
    vis_state: &mut VisualizingState,
    shrink_time: f64,
) {
    for circle in circles.iter_mut().filter(|c| !c.hit && !c.missed) {
        let time_since_spawn = elapsed - circle.spawn_time;

        if time_since_spawn > shrink_time {
            circle.missed = true;

            // Only record miss if not in no-fail mode
            if !vis_state.no_fail {
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
) {
    // Pre-compute pulse intensity once
    let pulse_intensity = 0.5 + (elapsed.sin() as f32) * 0.5;

    for circle in circles {
        let time_since_spawn = elapsed - circle.spawn_time;

        if (0.0..=shrink_time).contains(&time_since_spawn) && !circle.hit {
            // Shrink the circle with a smooth scaling effect
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
