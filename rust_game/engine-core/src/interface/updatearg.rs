use crate::Play;
use crate::frame_manager::{
    FrameInput,
    FrameState,
};
use crate::game_time::FrameIndex;
use crate::interface::InitialInformation;
use commons::time::TimeDuration;

#[derive(Debug)]
pub struct UpdateArg<'a, Game: Play> {
    initial_information: &'a InitialInformation<Game>,
    frame_index: FrameIndex,
    frame_state: &'a FrameState<Game::State>,
    frame_inputs: &'a Vec<FrameInput<Game::InputAggregate>>,
}

impl<'a, 'b, Game: Play> UpdateArg<'a, Game> {
    pub fn new(
        initial_information: &'a InitialInformation<Game>,
        step: FrameIndex,
        state: &'a FrameState<Game::State>,
        inputs: &'a Vec<FrameInput<Game::InputAggregate>>,
    ) -> Self {
        return Self {
            initial_information,
            frame_index: step,
            frame_state: state,
            frame_inputs: inputs,
        };
    }

    pub fn get_frame_input(&self, player_index: usize) -> &FrameInput<Game::InputAggregate> {
        return &self.frame_inputs[player_index];
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        return self.frame_index;
    }

    pub fn get_frame_state(&self) -> &'a FrameState<Game::State> {
        return self.frame_state;
    }

    //TODO: refactor to not use usize
    pub fn get_current_duration_since_start(&self) -> TimeDuration {
        return self
            .initial_information
            .get_server_config()
            .get_frame_duration()
            .duration_from_start(&self.frame_index);
    }

}
