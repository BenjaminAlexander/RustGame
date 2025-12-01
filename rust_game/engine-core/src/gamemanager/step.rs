use std::mem::take;

use log::warn;

use crate::game_time::FrameIndex;
use crate::interface::{
    GameTrait,
    UpdateArg,
};
use crate::messaging::InputMessage;
use crate::InitialInformation;

pub struct Step<Game: GameTrait> {
    frame_index: FrameIndex,
    state: StateHolder<Game::State>,
    inputs: Vec<Input<Game::ClientInput>>,
    input_count: usize,
    need_to_compute_next_state: bool,
}

#[derive(Clone, Debug, Default)]
pub enum Input<T> {
    /// Pending signifies that an input from a client isn't yet known but may
    /// become known in the future.
    #[default]
    Pending,

    /// The Input has been received from the client which is the authoritative source
    Authoritative(T),

    /// The client never submitted an input in a timely manner and the server
    /// has authoritatively decided that the client cannot submit an input in the future
    AuthoritativeMissing,
}

#[derive(Default)]
pub enum StateHolder<T> {

    //TODO: maybe get rid of the concept of an empty state holder
    #[default]
    None,
    Authoritative(T),
    NonAuthoritative(T),
}

impl<Game: GameTrait> Step<Game> {
    pub fn blank(step_index: FrameIndex, player_count: usize) -> Self {
        let inputs = vec![Input::Pending; player_count];

        return Self {
            frame_index: step_index,
            state: StateHolder::None,
            inputs,
            input_count: 0,
            need_to_compute_next_state: true,
        };
    }

    pub fn set_input(&mut self, input_message: InputMessage<Game>) {
        let index = input_message.get_player_index();

        //TODO: make a way for the server to say a input is missing
        //let x = &mut self.inputs[index];
        match self.inputs[index] {
            Input::Pending => {
                self.input_count = self.input_count + 1;
                self.inputs[index] = Input::Authoritative(input_message.get_input());
                self.need_to_compute_next_state = true;
            }
            Input::Authoritative(_) => {
                warn!("Received a duplicate input, ignorning it")
            }
            Input::AuthoritativeMissing => {
                warn!("Received a input where one has athoritatively been declared missing")
            }
        }
    }

    pub fn timeout_remaining_inputs(&mut self) {
        for input in &mut self.inputs {
            if let Input::Pending = input {
                self.input_count = self.input_count + 1;
                *input = Input::AuthoritativeMissing;
                self.need_to_compute_next_state = true;
                //TODO: send notification to clients
                warn!("Timing out a player input")
            }
        }
    }

    pub fn are_inputs_complete(&self) -> bool {
        self.input_count == self.inputs.len()
    }

    pub fn set_state(&mut self, state: Game::State, is_authoritative: bool) {

        if self.is_state_authoritative() {
            // No-op, ignore the new state if this one is already authoritative
            return;
        }

        self.state = if is_authoritative {
            StateHolder::Authoritative(state)
        } else {
            StateHolder::NonAuthoritative(state)
        };
        
        self.need_to_compute_next_state = true;
    }

    pub fn calculate_next_state(
        &mut self,
        initial_information: &InitialInformation<Game>,
    ) -> Option<(Game::State, bool)> {
        if !self.need_to_compute_next_state {
            return None;
        }

        let (state, is_authoritative) = match &self.state {
            StateHolder::None => {
                panic!("Tried to compute next state from a missing state");
            },
            StateHolder::Authoritative(state) => (state, true),
            StateHolder::NonAuthoritative(state) => (state, false),
        };

        let is_next_state_authoritative = self.are_inputs_complete() && is_authoritative;
        
        let arg = UpdateArg::new(initial_information, self.frame_index, state, &self.inputs);

        let next_state = Game::get_next_state(&arg);

        self.need_to_compute_next_state = false;

        Some((next_state, is_next_state_authoritative))
    }

    //TODO: rename
    pub fn get_step_index(&self) -> FrameIndex {
        return self.frame_index;
    }

    pub fn is_state_authoritative(&self) -> bool {
        match self.state {
            StateHolder::None => false,
            StateHolder::Authoritative(_) => true,
            StateHolder::NonAuthoritative(_) => false,
        }
    }
}
