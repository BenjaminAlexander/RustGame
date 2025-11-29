use crate::game_time::FrameIndex;
use crate::gamemanager::Input;
use crate::interface::game::GameTrait;
use crate::interface::InitialInformation;
use commons::time::TimeDuration;

#[derive(Debug)]
pub struct UpdateArg<'a, Game: GameTrait> {
    initial_information: &'a InitialInformation<Game>,
    //TODO: rename
    step: FrameIndex,
    state: &'a Game::State,
    inputs: &'a Vec<Input<Game::ClientInput>>,
}

impl<'a, 'b, Game: GameTrait> UpdateArg<'a, Game> {
    pub fn new(
        initial_information: &'a InitialInformation<Game>,
        step: FrameIndex,
        state: &'a Game::State,
        inputs: &'a Vec<Input<Game::ClientInput>>,
    ) -> Self {
        return Self {
            initial_information,
            step,
            state,
            inputs,
        };
    }

    pub fn get_input(&self, player_index: usize) -> &Input<Game::ClientInput> {
        return &self.inputs[player_index];
    }

    pub fn get_current_step(&self) -> FrameIndex {
        return self.step;
    }

    pub fn get_state(&self) -> &'a Game::State {
        return self.state;
    }

    pub fn get_next_step(&self) -> FrameIndex {
        return self.get_current_step() + 1;
    }

    //TODO: refactor to not use usize
    pub fn get_current_duration_since_start(&self) -> TimeDuration {
        return self
            .initial_information
            .get_server_config()
            .get_frame_duration()
            .get_frame_duration()
            .mul_f64(self.step.usize() as f64);
    }

    //TODO: refactor to not use usize
    pub fn get_next_step_duration_since_start(&self) -> TimeDuration {
        return self
            .initial_information
            .get_server_config()
            .get_frame_duration()
            .get_frame_duration()
            .mul_f64(self.get_next_step().usize() as f64);
    }
}
