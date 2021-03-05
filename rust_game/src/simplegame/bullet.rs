use crate::simplegame::{Vector2, STEP_DURATION, SimpleInput};
use serde::{Deserialize, Serialize};
use piston::{RenderArgs, ButtonState};
use opengl_graphics::{GlGraphics, OpenGL};
use graphics::{Context, rectangle};
use graphics::*;
use crate::gametime::{TimeDuration, EPOCH};
use log::{info, warn, trace};

const MAX_RANGE: f64 = 5000 as f64;

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Bullet {
    start_step: usize,
    start_position: Vector2,
    velocity: Vector2,
}

impl Bullet {
    pub fn new(start_step: usize, start_position: Vector2, aim_point: Vector2) -> Self {

        let velocity = (aim_point - start_position).normalize();

        return Self{
            start_step,
            start_position,
            velocity,
        };
    }

    pub fn get_position(&self, duration_since_game_start: TimeDuration) -> Option<Vector2> {
        let duration_since_bullet_start = duration_since_game_start - (STEP_DURATION * self.start_step as i64);
        if duration_since_bullet_start.get_millis() >= 0 {
            return Some(self.start_position + self.velocity * duration_since_bullet_start.get_millis() as f64);
        } else {
            return None;
        }
    }

    pub fn should_remove(&self, duration_since_game_start: TimeDuration) -> bool {
        if let Some(current_position) = self.get_position(duration_since_game_start) {
            return (current_position -  self.start_position).get_length() > MAX_RANGE;
        } else {
            return false;
        }

    }

    pub fn draw(&self, duration_since_game_start: TimeDuration, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        if let Some(current_position) = self.get_position(duration_since_game_start) {

            let (x, y) = current_position.get();
            let x_in_window = (x as f64 / args.draw_size[0] as f64) * args.window_size[0];
            let y_in_window = (y as f64 / args.draw_size[1] as f64) * args.window_size[1];

            let square = rectangle::square(0.0, 0.0, 10.0);
            let rotation = 0 as f64;

            let transform = context
                .transform
                .trans(x_in_window, y_in_window)
                .rot_rad(rotation)
                .trans(-5.0, -5.0);

            rectangle(BLUE, square, transform, gl);
        }
    }
}