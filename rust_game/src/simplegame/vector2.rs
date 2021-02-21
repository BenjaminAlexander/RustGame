use serde::{Deserialize, Serialize};

use crate::interface::{Input, State, InputEvent};
use std::ops::{Sub, Add, Mul};
use num::traits::Pow;
use num::traits::real::Real;
use std::hash::{Hash, Hasher};

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

    pub fn get_length(&self) -> f64 {
        let power_sum: f64 = self.x.pow(2) + self.y.pow(2);
        return power_sum.sqrt();
    }

    pub fn normalize(&self) -> Self {
        let length = self.get_length();
        if length == 0 as f64 {
            return Self::zero();
        } else {
            return Self::new(self.x / length, self.y / length);
        }
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

impl<T> Mul<T> for Vector2
    where f64 : Mul<T, Output=f64>,
          T: Copy {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        return Self {
            x: self.x * rhs,
            y: self.y * rhs,
        };
    }
}

impl InputEvent for Vector2 {

}

impl Hash for Vector2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_be_bytes().hash(state);
        self.y.to_be_bytes().hash(state);
    }
}