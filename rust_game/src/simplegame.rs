pub use crate::simplegame::vector2::*;
use crate::interface::{Input, State};
use serde::{Deserialize, Serialize};

mod vector2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleState {
    pub vectors: Vec<Vector2>
}

impl SimpleState {
    pub fn new(player_count: usize) -> Self {
        let mut new = Self{vectors: Vec::new()};
        for i in 0..player_count {
            new.vectors.push(Vector2::new(0 as f32, 0 as f32));
        }
        return new;
    }
}

impl State<SimpleInput, Vector2> for SimpleState {
    fn get_next_state(&self, inputs: &Vec<Option<SimpleInput>>) -> Self {
        let mut new = self.clone();

        for i in 0..new.vectors.len() {
            if let Some(Some(input)) = inputs.get(i) {
                if let Some(vector) = &input.vector_option {
                    new.vectors[i].set(vector);
                }
            }
        }

        return new;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleInput {
    vector_option: Option<Vector2>
}

impl Input<Vector2> for SimpleInput {

    fn new() -> Self {
        return Self{vector_option: None};
    }

    fn accumulate(&mut self, input_event: Vector2) {
        self.vector_option = Some(input_event);
    }
}