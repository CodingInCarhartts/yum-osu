use crate::constants::*;
use crate::structs::{Circle, FloatingText};
use macroquad::prelude::{draw_circle, is_key_pressed, mouse_position, Color, KeyCode, Vec2};
use rand::Rng;

/// Initialize circles for a game with animations
pub fn initialize_circles(
    beats: &[f64],
    rng: &mut impl Rng,
    spawn_radius: f32,
    center: Vec2,
    shrink_time: f64,
    delay: f64,
) -> Vec<Circle> {
    let mut circles = Vec::with_capacity(beats.len());

    for &beat_time in beats {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(0.0..spawn_radius);

        let position = Vec2::new(
            center.x + distance * angle.cos(),
            center.y + distance * angle.sin(),
        );

        circles.push(Circle {
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

/// Handle key hits with animation and feedback
pub fn handle_key_hits(circles: &mut Vec<Circle>, elapsed: f64, score: &mut i32, shrink_time: f64) {
    let mouse_pos: Vec2 = mouse_position().into();
    let key_pressed = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::S);

    for circle in circles.iter_mut().filter(|c| !c.hit) {
        if let Some(radius) = circle_radius(circle, elapsed, shrink_time) {
            if mouse_pos.distance(circle.position) < radius && key_pressed {
                circle.hit = true;
                *score += calculate_score(circle.hit_time, elapsed);
                break;
            }
        }
    }
}

/// Calculate the shrinking radius with animation
fn circle_radius(circle: &Circle, elapsed: f64, shrink_time: f64) -> Option<f32> {
    let time_since_spawn = elapsed - circle.spawn_time;
    if (0.0..=shrink_time).contains(&time_since_spawn) {
        Some(circle.max_radius * (1.0 - ((time_since_spawn / shrink_time) as f32)))
    } else {
        None
    }
}

/// Calculate the spawn radius based on the screen size
pub fn calculate_spawn_radius(width: f32, height: f32) -> f32 {
    width.min(height) / 2.0 - 100.0
}

/// Handle missed circles and animate a "Miss" text
pub fn handle_missed_circles(
    circles: &mut Vec<Circle>,
    elapsed: f64,
    floating_texts: &mut Vec<FloatingText>,
    shrink_time: f64,
) {
    for circle in circles.iter_mut().filter(|c| !c.hit && !c.missed) {
        let time_since_spawn = elapsed - circle.spawn_time;

        if time_since_spawn > shrink_time {
            circle.missed = true;

            floating_texts.push(FloatingText {
                text: "Miss".to_string(),
                position: circle.position,
                spawn_time: elapsed,
                duration: 1.0,
            });
        }
    }
}

/// Score calculation based on the hit time and elapsed time
pub fn calculate_score(hit_time: f64, current_time: f64) -> i32 {
    let time_difference = (current_time - hit_time).abs();
    if time_difference < 0.1 {
        300
    } else if time_difference < 0.3 {
        100
    } else {
        50
    }
}

/// Draw animated circles with stylizing and dynamic color transitions
pub fn draw_circles(circles: &Vec<Circle>, elapsed: f64, shrink_time: f64) {
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

            // Draw an animated outline with a pulsing effect
            draw_circle(
                circle.position.x,
                circle.position.y,
                radius + OUTLINE_THICKNESS,
                Color::new(
                    OUTLINE_COLOR.r,
                    OUTLINE_COLOR.g,
                    OUTLINE_COLOR.b,
                    pulse_intensity,
                ),
            );

            // Use a predefined neon color for the circle's fill
            let color = Color::new(0.0, 0.75, 1.0, alpha);

            draw_circle(circle.position.x, circle.position.y, radius, color);
        }
    }
}
