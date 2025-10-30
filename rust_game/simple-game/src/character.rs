use crate::bullet::Bullet;
use crate::simplegameimpl::SimpleGameImpl;
use commons::geometry::twod::Vector2;
use commons::time::TimeDuration;
use engine_core::{
    ClientUpdateArg,
    GameTrait,
};
use graphics::math::Matrix2d;
use graphics::*;
use graphics::{
    rectangle,
    Context,
};
use opengl_graphics::GlGraphics;
use piston::RenderArgs;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    player_index: usize,
    velocity: Vector2,
    position: Vector2,
    health: u8,
}

impl Character {
    pub fn new(player_index: usize, position: Vector2) -> Self {
        return Self {
            player_index,
            velocity: Vector2::new(0 as f64, 0 as f64),
            position,
            health: 10,
        };
    }

    pub fn get_player_index(&self) -> usize {
        return self.player_index;
    }

    pub fn get_position(&self) -> &Vector2 {
        return &self.position;
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn is_hit(&self, bullet: &Bullet, duration_since_start: TimeDuration) -> bool {
        if let Some(bullet_position) = bullet.get_position(duration_since_start) {
            if (bullet_position - self.position).get_length() < 75.0 {
                return true;
            }
        }
        return false;
    }

    pub fn reduce_health(&mut self) {
        if self.health > 0 {
            self.health = self.health - 1;
        }
    }

    pub fn move_character(&mut self, arg: &ClientUpdateArg<SimpleGameImpl>) {
        if let Some(input) = arg.get_input(self.player_index) {
            self.velocity = input.get_velocity();
        }

        self.position =
            self.position + self.velocity * SimpleGameImpl::STEP_PERIOD.as_secs_f64() * 500.0;
    }

    pub fn get_fired_bullet(&self, arg: &ClientUpdateArg<SimpleGameImpl>) -> Option<Bullet> {
        if let Some(input) = arg.get_input(self.player_index) {
            if input.should_fire() {
                return Some(Bullet::new(
                    arg.get_current_step(),
                    self.position,
                    input.get_aim_point(),
                ));
            }
        }

        return None;
    }

    pub fn draw(&self, args: &RenderArgs, transform: Matrix2d, gl: &mut GlGraphics, local_player_index: usize) {
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        let (x, y) = self.position.get();
        let x_in_window = x;//(x as f64 / ;
        let y_in_window = y;//(y as f64 / args.draw_size[1] as f64) * args.window_size[1];


        if local_player_index == self.player_index {
            let square = rectangle::square(0.0, 0.0, 54.0);
            let rotation = 1 as f64;

            let local_player_transform = transform
                .trans(x_in_window, y_in_window)
                //.rot_rad(rotation)
                .trans(-27.0, -27.0);

            rectangle(GREEN, square, local_player_transform, gl);
        }

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = 0 as f64;

        let player_transform = transform
            .trans(x_in_window, y_in_window)
            //.rot_rad(rotation)
            .trans(-25.0, -25.0);

        rectangle(RED, square, player_transform, gl);

        //draw health bar
        let base = self.player_index as f64 * 10.0;
        let health_rectangle =
            rectangle::rectangle_by_corners(0.0, base, 10.0 * self.health as f64, base + 10.0);
        rectangle(RED, health_rectangle, transform, gl);
    }
}
