use crate::simplegame::{Vector2, SimpleInputEvent};
use crate::interface::Input;
use piston::{Motion, ButtonArgs, Button, Key, ButtonState};
use piston::input::Input as PistonInput;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleInput {
    aim_point: Vector2,
    velocity: Vector2,
    should_fire: bool,
}

impl SimpleInput {

    pub fn new(aim_point: Vector2, velocity: Vector2, should_fire: bool) -> Self {
        return Self{
            aim_point,
            velocity,
            should_fire
        };
    }

    pub fn get_aim_point(&self) -> Vector2 {
        return self.aim_point;
    }

    pub fn get_velocity(&self) -> Vector2 {
        return self.velocity;
    }

    pub fn should_fire(&self) -> bool {
        return self.should_fire;
    }
}

impl Input for SimpleInput {

}