use crate::interface::GameTrait;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct StepMessage<GameType: GameTrait> {
    step_index: usize,
    state: GameType::StateType
}

impl<GameType: GameTrait> StepMessage<GameType> {

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

impl<GameType: GameTrait> Clone for StepMessage<GameType> {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            state: self.state.clone()
        }
    }
}

impl<GameType: GameTrait> PartialEq for StepMessage<GameType> {

    fn eq(&self, other: &Self) -> bool {
        return self.step_index.eq(&other.step_index);
    }
}

impl<GameType: GameTrait> Eq for StepMessage<GameType> {

}

impl<GameType: GameTrait> PartialOrd for StepMessage<GameType> {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<GameType: GameTrait> Ord for StepMessage<GameType> {

    fn cmp(&self, other: &Self) -> Ordering {
        return self.step_index.cmp(&other.step_index);
    }
}