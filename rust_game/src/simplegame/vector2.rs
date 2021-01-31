use serde::{Deserialize, Serialize};

use crate::interface::{Input, State, InputEvent};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector2 {
    x: f32,
    y: f32,
}

impl State<Vector2, Vector2> for Vector2 {
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

impl Input<Vector2> for Vector2 {
    fn new() -> Self {
        Vector2::new(0 as f32, 0 as f32)
    }

    fn accumulate(&mut self, input_event: Vector2) {
        self.x = self.x + input_event.x;
        self.y = self.y + input_event.y;
    }
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

impl InputEvent for Vector2 {

}