use crate::SimpleGameImpl;
use commons::geometry::twod::Vector2;
use commons::time::TimeDuration;
use engine_core::GameTrait;
use graphics::rectangle;
use graphics::*;
use opengl_graphics::GlGraphics;
use serde::{
    Deserialize,
    Serialize,
};

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

        return Self {
            start_step,
            start_position,
            velocity,
        };
    }

    pub fn get_position(&self, duration_since_game_start: TimeDuration) -> Option<Vector2> {
        let duration_since_bullet_start = duration_since_game_start
            - &SimpleGameImpl::STEP_PERIOD.mul_f64(self.start_step as f64);
        if duration_since_bullet_start.as_secs_f64() >= 0.0 {
            //TODO: move div by 1000
            return Some(
                self.start_position
                    + self.velocity * (duration_since_bullet_start.as_secs_f64() * 1000.0),
            );
        } else {
            return None;
        }
    }

    pub fn should_remove(&self, duration_since_game_start: TimeDuration) -> bool {
        if let Some(current_position) = self.get_position(duration_since_game_start) {
            return (current_position - self.start_position).get_length() > MAX_RANGE;
        } else {
            return false;
        }
    }

    pub fn draw(
        &self,
        duration_since_game_start: TimeDuration,
        context: Context,
        gl: &mut GlGraphics,
    ) {
        const BULLET_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const BULLET_SIZE: f64 = 10.0;

        if let Some(current_position) = self.get_position(duration_since_game_start) {
            let (x, y) = current_position.get();

            let square = rectangle::square(0.0, 0.0, BULLET_SIZE);
            let rotation = 0 as f64;

            let bullet_transform = context
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-0.5 * BULLET_SIZE, -0.5 * BULLET_SIZE);

            rectangle(BULLET_COLOR, square, bullet_transform, gl);
        }
    }
}
