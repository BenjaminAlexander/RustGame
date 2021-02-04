use serde::{Deserialize, Serialize};

use crate::interface::{Input, State, InputEvent};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0 as f64, y: 0 as f64 }
    }

    pub fn set(&mut self, other: &Vector2) {
        self.x = other.x;
        self.y = other.y;
    }

    pub fn set_x(&mut self, value: f64) {
        self.x = value;
    }

    pub fn set_y(&mut self, value: f64) {
        self.y = value;
    }

    pub fn get_x(&mut self) -> f64 {
        return self.x;
    }

    pub fn get_y(&mut self) -> f64 {
        return self.y;
    }

    pub fn get(&self) -> (f64, f64) {
        return (self.x, self.y);
    }

    pub fn add(&self, other: Vector2) -> Vector2 {
        Vector2::new(self.x + other.x, self.y + other.y)
    }

    pub fn multiply(&self, value: f64) -> Vector2 {
        return Vector2::new(self.x * value, self.y * value);
    }
}

impl InputEvent for Vector2 {

}