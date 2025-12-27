use crate::Play;
use crate::frame_manager::{
    FrameInput,
    FrameState,
};
use crate::game_time::FrameIndex;
use crate::interface::InitialInformation;
use commons::time::TimeDuration;

/// The current frame from which to calculate the next frame
#[derive(Debug)]
pub struct CurrentFrame<'a, Game: Play> {
    initial_information: &'a InitialInformation<Game>,
    frame_index: FrameIndex,
    frame_state: &'a FrameState<Game::State>,
    frame_inputs: &'a Vec<FrameInput<Game::InputAggregate>>,
}

impl<'a, 'b, Game: Play> CurrentFrame<'a, Game> {
    pub(crate) fn new(
        initial_information: &'a InitialInformation<Game>,
        frame_index: FrameIndex,
        frame_state: &'a FrameState<Game::State>,
        frame_inputs: &'a Vec<FrameInput<Game::InputAggregate>>,
    ) -> Self {
        return Self {
            initial_information,
            frame_index,
            frame_state,
            frame_inputs,
        };
    }

    /// Returns the [FrameInput] for a player
    pub fn get_frame_input(&self, player_index: usize) -> &'a FrameInput<Game::InputAggregate> {
        return &self.frame_inputs[player_index];
    }

    /// Returns the [FrameIndex] of this frame
    pub fn get_frame_index(&self) -> FrameIndex {
        return self.frame_index;
    }

    /// Returns the [FrameState]
    pub fn get_frame_state(&self) -> &'a FrameState<Game::State> {
        return self.frame_state;
    }

    /// Returns the [InitialInformation] of the game
    pub fn get_initial_information(&self) -> &'a InitialInformation<Game> {
        return self.initial_information;
    }

    /// Returns the [TimeDuration] between the start of the game and the current frame
    pub fn get_current_frame_duration_since_start(&self) -> TimeDuration {
        return self
            .initial_information
            .get_server_config()
            .get_frame_duration()
            .duration_from_start(&self.frame_index);
    }

    /// Returns the [TimeDuration] between the start of the game and the next frame
    pub fn get_next_frame_duration_since_start(&self) -> TimeDuration {
        return self
            .initial_information
            .get_server_config()
            .get_frame_duration()
            .duration_from_start(&self.frame_index.next());
    }
}
