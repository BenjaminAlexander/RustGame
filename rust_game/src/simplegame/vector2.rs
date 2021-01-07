use serde::{Deserialize, Serialize};

use crate::interface::{Input, State};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector2 {
    x: f32,
    y: f32,
}

impl State<Vector2> for Vector2 {
    fn get_next_state(&self, inputs: &Vec<Option<Vector2>>) -> Self {
        let mut new_vector = self.clone();
        for input in inputs {
            match input.as_ref() {
                None => {}
                Some(input_vector) => {
                    new_vector = new_vector.add(input_vector)
                }
            }
        }

        return new_vector;
    }
}

impl Input for Vector2 {

}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    // pub fn get_x(&self) -> f32 {
    //     self.x
    // }
    //
    // pub fn get_y(&self) -> f32 {
    //     self.y
    // }

    pub fn add(&self, other: &Vector2) -> Vector2 {
        Vector2::new(self.x + other.x, self.y + other.y)
    }
}