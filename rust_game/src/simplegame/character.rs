use crate::simplegame::{Vector2, STEP_DURATION, SimpleInput, SimpleState};
use serde::{Deserialize, Serialize};
use piston::{RenderArgs, ButtonState};
use opengl_graphics::{GlGraphics, OpenGL};
use graphics::{Context, rectangle};
use graphics::*;
use crate::simplegame::bullet::Bullet;
use log::{warn, trace, info};
use crate::interface::NextStateArg;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    player_index: usize,
    velocity: Vector2,
    position: Vector2
}

impl Character {
    pub fn new(player_index: usize, position: Vector2) -> Self {
        return Self{
            player_index,
            velocity: Vector2::new(0 as f64, 0 as f64),
            position
        };
    }

    pub fn get_position(&self) -> &Vector2 {
        return &self.position;
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn move_character(&mut self, arg: &NextStateArg<SimpleInput>) {

        if let Some(input) = arg.get_input(self.player_index) {
            self.velocity = input.get_velocity();
        }

        self.position = self.position + self.velocity * STEP_DURATION.get_millis() as f64 * 0.5;
    }

    pub fn get_fired_bullet(&self, arg: &NextStateArg<SimpleInput>) -> Option<Bullet> {
        if let Some(input) = arg.get_input(self.player_index) {
            if input.should_fire() {
                return Some(Bullet::new(
                    arg.get_current_step(),
                    self.position,
                    input.get_aim_point()
                ));
            }
        }

        return None;
    }

    pub fn draw(&self, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let (x, y) = self.position.get();
        let x_in_window = (x as f64 / args.draw_size[0] as f64) * args.window_size[0];
        let y_in_window = (y as f64 / args.draw_size[1] as f64) * args.window_size[1];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = 0 as f64;

        let transform = context
            .transform
            .trans(x_in_window, y_in_window)
            .rot_rad(rotation)
            .trans(-25.0, -25.0);

        rectangle(RED, square, transform, gl);
    }
}