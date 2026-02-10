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
pub fn handle_key_hits(
    circles: &mut Vec<Circle>,
    elapsed: f64,
    vis_state: &mut VisualizingState,
    shrink_time: f64,
    config: &GameConfig,
) {
    let mouse_pos: Vec2 = mouse_position().into();

    // Check key presses using configured keys
    let primary_pressed = is_key_pressed(config.key_bindings.primary_hit_key());
    let secondary_pressed = is_key_pressed(config.key_bindings.secondary_hit_key());
    let key_pressed = primary_pressed || secondary_pressed;

    if !key_pressed {
        return;
    }

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

/// Calculate score from timing difference
fn calculate_score_from_timing(time_difference: f64) -> i32 {
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

/// Score calculation based on the hit time and elapsed time (legacy)
pub fn calculate_score(hit_time: f64, current_time: f64) -> i32 {
    let time_difference = (current_time - hit_time).abs();
    calculate_score_from_timing(time_difference)
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

            // Draw approach circle
            draw_circle_lines(
                circle.position.x,
                circle.position.y,
                radius,
                2.0,
                Color::new(
                    circle_color.r,
                    circle_color.g,
                    circle_color.b,
                    0.3 + pulse_intensity * 0.3,
                ),
            );
        }
    }
}

/// Draw circle outline
fn draw_circle_lines(x: f32, y: f32, radius: f32, thickness: f32, color: Color) {
    let segments = 32;
    let angle_step = std::f32::consts::TAU / segments as f32;

    for i in 0..segments {
        let angle1 = i as f32 * angle_step;
        let angle2 = ((i + 1) % segments) as f32 * angle_step;

        let x1 = x + radius * angle1.cos();
        let y1 = y + radius * angle1.sin();
        let x2 = x + radius * angle2.cos();
        let y2 = y + radius * angle2.sin();

        macroquad::shapes::draw_line(x1, y1, x2, y2, thickness, color);
    }
}
