use crate::interface::Game;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct StepMessage<GameType: Game> {
    step_index: usize,
    state: GameType::StateType
}

impl<GameType: Game> StepMessage<GameType> {

    pub fn new(step_index: usize, state: GameType::StateType) -> Self {
        Self{step_index, state}
    }

    pub fn get_step_index(&self) -> usize {
        self.step_index
    }

    pub fn get_state(&self) -> &GameType::StateType {
        &self.state
    }
}

impl<GameType: Game> Clone for StepMessage<GameType> {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            state: self.state.clone()
        }
    }
}

impl<GameType: Game> PartialEq for StepMessage<GameType> {

    fn eq(&self, other: &Self) -> bool {
        return self.step_index.eq(&other.step_index);
    }
}

impl<GameType: Game> Eq for StepMessage<GameType> {

}

impl<GameType: Game> PartialOrd for StepMessage<GameType> {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<GameType: Game> Ord for StepMessage<GameType> {

    fn cmp(&self, other: &Self) -> Ordering {
        return self.step_index.cmp(&other.step_index);
    }
}