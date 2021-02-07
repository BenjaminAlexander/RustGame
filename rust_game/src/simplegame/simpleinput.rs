use crate::simplegame::{Vector2, SimpleInputEvent};
use crate::interface::Input;
use piston::{Motion, ButtonArgs, Button, Key, ButtonState};
use piston::input::Input as PistonInput;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleInput {
    vector_option: Option<Vector2>,
    velocity: Vector2
}

impl SimpleInput {

    pub fn new(vector_option: Option<Vector2>, velocity: Vector2) -> Self {
        return Self{
            vector_option,
            velocity
        };
    }

    pub fn get_vector_option(&self) -> Option<Vector2> {
        return self.vector_option;
    }

    pub fn get_velocity(&self) -> Vector2 {
        return self.velocity;
    }
}

impl Input for SimpleInput {

}