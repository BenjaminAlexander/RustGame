use crate::interface::GameTrait;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct StepMessage<Game: GameTrait> {
    step_index: usize,
    state: Game::State
}

impl<Game: GameTrait> StepMessage<Game> {

    pub fn new(step_index: usize, state: Game::State) -> Self {
        Self{step_index, state}
    }

    pub fn get_step_index(&self) -> usize {
        self.step_index
    }

    pub fn get_state(&self) -> &Game::State {
        &self.state
    }
}

impl<Game: GameTrait> Clone for StepMessage<Game> {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            state: self.state.clone()
        }
    }
}

impl<Game: GameTrait> PartialEq for StepMessage<Game> {

    fn eq(&self, other: &Self) -> bool {
        return self.step_index.eq(&other.step_index);
    }
}

impl<Game: GameTrait> Eq for StepMessage<Game> {

}

impl<Game: GameTrait> PartialOrd for StepMessage<Game> {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<Game: GameTrait> Ord for StepMessage<Game> {

    fn cmp(&self, other: &Self) -> Ordering {
        return self.step_index.cmp(&other.step_index);
    }
}