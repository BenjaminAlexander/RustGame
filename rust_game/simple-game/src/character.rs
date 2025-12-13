use crate::bullet::Bullet;
use crate::simplegameimpl::SimpleGameImpl;
use commons::geometry::twod::Vector2;
use commons::time::TimeDuration;
use engine_core::{
    GameTrait,
    UpdateArg,
};
use graphics::rectangle;
use graphics::*;
use opengl_graphics::GlGraphics;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    player_index: usize,
    position: Vector2,
    health: u8,
}

impl Character {
    pub fn new(player_index: usize, position: Vector2) -> Self {
        return Self {
            player_index,
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

    pub fn move_character(&mut self, arg: &UpdateArg<SimpleGameImpl>) {
        if let Some(input) = arg.get_input(self.player_index).input() {
            let velocity = input.get_velocity();

            self.position = self.position + velocity * SimpleGameImpl::STEP_PERIOD.as_secs_f64() * 500.0;
        }
    }

    pub fn get_fired_bullet(&self, arg: &UpdateArg<SimpleGameImpl>) -> Option<Bullet> {
        if let Some(input) = arg.get_input(self.player_index).input() {
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

    pub fn draw(&self, context: Context, gl: &mut GlGraphics, local_player_index: usize) {
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        const CHARACTER_SIZE: f64 = 50.0;
        const HEALTH_BAR_HEIGHT: f64 = 10.0;
        const HEALTH_BAR_TOKEN_WIDTH: f64 = 10.0;

        let is_local_player = local_player_index == self.player_index;

        let color = if self.health > 0 {
            if is_local_player {
                BLUE
            } else {
                RED
            }
        } else {
            GREEN
        };

        let (x, y) = self.position.get();

        let square = rectangle::square(0.0, 0.0, CHARACTER_SIZE);

        let player_transform = context
            .transform
            .trans(x, y)
            .trans(-0.5 * CHARACTER_SIZE, -0.5 * CHARACTER_SIZE);

        rectangle(color, square, player_transform, gl);

        //draw health bar
        if is_local_player {
            let health_rectangle = rectangle::rectangle_by_corners(
                0.0,
                0.0,
                HEALTH_BAR_TOKEN_WIDTH * self.health as f64,
                HEALTH_BAR_HEIGHT,
            );
            rectangle(RED, health_rectangle, context.transform, gl);
        }
    }
}
