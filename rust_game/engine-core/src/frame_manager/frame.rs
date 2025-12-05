use log::warn;

use crate::game_time::FrameIndex;
use crate::frame_manager::ObserveFrames;
use crate::interface::{
    GameTrait,
    UpdateArg,
};
use crate::InitialInformation;
use crate::messaging::StateMessage;

pub struct Frame<Game: GameTrait> {
    frame_index: FrameIndex,
    state: State<Game::State>,
    inputs: Vec<Input<Game::ClientInput>>,
    authoritative_input_count: usize,
    need_to_compute_next_state: bool,
}

#[derive(Clone, Debug, Default)]
pub enum Input<T> {
    /// Pending signifies that an input from a client isn't yet known but may
    /// become known in the future.
    #[default]
    Pending,

    /// The Input has been received from the local client.  This input can still 
    /// potentially be rejected by the server if the client's message is dropped 
    /// or is late.
    NonAuthoritative(T),

    /// The Input has been received from the sever as authoritative
    Authoritative(T),

    /// The client never submitted an input in a timely manner and the server
    /// has authoritatively decided that the client cannot submit an input in the future
    AuthoritativeMissing,
}

impl<T> Input<T> {
    pub fn is_authoritative(&self) -> bool {
        match self {
            Input::Pending => false,
            Input::NonAuthoritative(_) => false,
            Input::Authoritative(_) => true,
            Input::AuthoritativeMissing => true,
        }
    }
}

#[derive(Default)]
pub enum State<T> {
    #[default]
    None,
    Authoritative(T),
    NonAuthoritative(T),
}

impl<Game: GameTrait> Frame<Game> {
    pub fn blank(step_index: FrameIndex, player_count: usize) -> Self {
        let inputs = vec![Input::Pending; player_count];

        return Self {
            frame_index: step_index,
            state: State::None,
            inputs,
            authoritative_input_count: 0,
            need_to_compute_next_state: true,
        };
    }

    pub fn set_input(&mut self, player_index: usize, input: Input<Game::ClientInput>) {
        let current_input = &mut self.inputs[player_index];

        if current_input.is_authoritative() {
            warn!("Received a duplicate input where an authoritative one has already been received, ignorning it");
            return;
        }

        if input.is_authoritative() {
            self.authoritative_input_count = self.authoritative_input_count + 1;
        }

        *current_input = input;
        self.need_to_compute_next_state = true;
    }

    pub fn timeout_remaining_inputs(&mut self, observer: &impl ObserveFrames<Game = Game>) {
        for (player_index, input) in &mut self.inputs.iter_mut().enumerate() {
            if let Input::Pending = input {
                self.authoritative_input_count = self.authoritative_input_count + 1;
                *input = Input::AuthoritativeMissing;
                self.need_to_compute_next_state = true;

                //TODO: Make a way to timeout all of a player's inputs immediatly when they disconnect.
                observer.on_input_authoritatively_missing(self.frame_index, player_index);
            }
        }
    }

    pub fn are_inputs_complete(&self) -> bool {
        self.authoritative_input_count == self.inputs.len()
    }

    pub fn set_state(&mut self, state: Game::State, is_authoritative: bool, observer: &impl ObserveFrames<Game = Game>) {

        if self.is_state_authoritative() {
            // No-op, ignore the new state if this one is already authoritative
            return;
        }

        self.state = if is_authoritative {
            State::Authoritative(state.clone())
        } else {
            State::NonAuthoritative(state.clone())
        };
        
        self.need_to_compute_next_state = true;

        observer.on_step_message(is_authoritative, StateMessage::new(self.frame_index, state));
    }

    pub fn calculate_next_state(
        &mut self,
        initial_information: &InitialInformation<Game>,
    ) -> Option<(Game::State, bool)> {
        if !self.need_to_compute_next_state {
            return None;
        }

        let (state, is_authoritative) = match &self.state {
            State::None => {
                panic!("Tried to compute next state from a missing state");
            },
            State::Authoritative(state) => (state, true),
            State::NonAuthoritative(state) => (state, false),
        };

        let is_next_state_authoritative = self.are_inputs_complete() && is_authoritative;
        
        let arg = UpdateArg::new(initial_information, self.frame_index, state, &self.inputs);

        let next_state = Game::get_next_state(&arg);

        self.need_to_compute_next_state = false;

        Some((next_state, is_next_state_authoritative))
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        return self.frame_index;
    }

    pub fn is_state_authoritative(&self) -> bool {
        match self.state {
            State::None => false,
            State::Authoritative(_) => true,
            State::NonAuthoritative(_) => false,
        }
    }
}
