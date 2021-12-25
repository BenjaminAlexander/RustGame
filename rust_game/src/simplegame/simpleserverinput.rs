use crate::interface::ServerInput;
use serde::{Deserialize, Serialize};
use crate::simplegame::SimpleState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleServerInput {
    events: Vec<SimplServerInputEvent>
}

impl SimpleServerInput {
    pub fn new() -> Self {
        return Self{
            events: Vec::new()
        };
    }

    pub fn add_event(&mut self, event: SimplServerInputEvent) {
        self.events.push(event);
    }

    pub fn apply_to_state(&self, state: &mut SimpleState) {
        for event in &self.events {
            event.apply_to_state(state);
        }
    }
}

impl ServerInput for SimpleServerInput {

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SimplServerInputEvent {
    CharacterHit{index: usize}
}

impl SimplServerInputEvent {
    pub fn apply_to_state(&self, state: &mut SimpleState) {
        match self {
            SimplServerInputEvent::CharacterHit {index} => state.hit_character(*index)
        }
    }
}