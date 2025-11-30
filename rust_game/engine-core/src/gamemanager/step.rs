use log::warn;

use crate::game_time::FrameIndex;
use crate::interface::{
    GameTrait,
    UpdateArg,
};
use crate::messaging::{
    InputMessage,
    StateMessage,
};
use crate::InitialInformation;

pub struct Step<Game: GameTrait> {
    frame_index: FrameIndex,
    state: StateHolder<Game>,
    inputs: Vec<Input<Game::ClientInput>>,
    input_count: usize,
    need_to_compute_next_state: bool,
}

#[derive(Clone, Debug)]
pub enum Input<T> {
    /// Pending signifies that an input from a client isn't yet known but may
    /// become known in the future.
    Pending,

    /// The Input has been received from the client which is the authoritative source
    Authoritative(T),

    /// The client never submitted an input in a timely manner and the server
    /// has authoritatively decided that the client cannot submit an input in the future
    AuthoritativeMissing,
}

pub enum StateHolder<Game: GameTrait> {
    None,
    Deserialized {
        state: Game::State,
        need_to_send_as_changed: bool,
    },
    ComputedIncomplete {
        state: Game::State,
        need_to_send_as_changed: bool,
    },
    ComputedComplete {
        state: Game::State,
        need_to_send_as_changed: bool,
        need_to_send_as_complete: bool,
    },
}

impl<Game: GameTrait> Step<Game> {
    pub fn blank(step_index: FrameIndex, player_count: usize) -> Self {
        let inputs = vec![Input::Pending; player_count];

        return Self {
            frame_index: step_index,
            state: StateHolder::None,
            inputs,
            input_count: 0,
            need_to_compute_next_state: false,
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

    //TODO: smells like this method should be somewhere else
    pub fn are_inputs_complete(&self, initial_information: &InitialInformation<Game>) -> bool {
        return match self.state {
            StateHolder::Deserialized { .. } => true,
            StateHolder::ComputedComplete { .. } => true,
            _ => false,
        } && self.input_count == initial_information.get_player_count();
    }

    pub fn set_final_state(&mut self, state_message: StateMessage<Game>) {
        let new_state = state_message.take_state();
        let mut has_changed = false;

        if let Some(old_state) = match &self.state {
            StateHolder::None => None,
            StateHolder::Deserialized { state, .. } => Some(state),
            StateHolder::ComputedIncomplete { state, .. } => Some(state),
            StateHolder::ComputedComplete { state, .. } => Some(state),
        } {
            let old_buf = rmp_serde::to_vec(old_state).unwrap();
            let new_buf = rmp_serde::to_vec(&new_state).unwrap();

            if old_buf.len() != new_buf.len() {
                has_changed = true;
            } else {
                for i in 0..old_buf.len() {
                    if !old_buf[i].eq(&new_buf[i]) {
                        has_changed = true;
                        break;
                    }
                }
            }
        } else {
            has_changed = true;
        }

        self.state = StateHolder::Deserialized {
            state: new_state,
            need_to_send_as_changed: has_changed,
        };

        if has_changed {
            self.need_to_compute_next_state = true;
        }

        //info!("Set final Step: {:?}", self.step_index);
    }

    pub fn calculate_next_state(
        &self,
        initial_information: &InitialInformation<Game>,
    ) -> StateHolder<Game> {
        if let Some(state) = match &self.state {
            StateHolder::None => None,
            StateHolder::Deserialized { state, .. } => Some(state),
            StateHolder::ComputedIncomplete { state, .. } => Some(state),
            StateHolder::ComputedComplete { state, .. } => Some(state),
        } {
            let arg = UpdateArg::new(initial_information, self.frame_index, state, &self.inputs);

            let next_state = Game::get_next_state(&arg);

            if self.are_inputs_complete(initial_information) {
                return StateHolder::ComputedComplete {
                    state: next_state,
                    need_to_send_as_changed: true,
                    need_to_send_as_complete: true,
                };
            } else {
                return StateHolder::ComputedIncomplete {
                    state: next_state,
                    need_to_send_as_changed: true,
                };
            }
        } else {
            return StateHolder::None;
        }
    }

    pub fn need_to_compute_next_state(&self) -> bool {
        return self.need_to_compute_next_state;
    }

    pub fn mark_as_calculation_not_needed(&mut self) {
        self.need_to_compute_next_state = false;
    }

    pub fn set_calculated_state(&mut self, state_holder: StateHolder<Game>) {
        self.need_to_compute_next_state = true;
        self.state = state_holder;
    }

    pub fn mark_as_complete(&mut self) {
        if let StateHolder::ComputedIncomplete {
            state,
            need_to_send_as_changed,
        } = &self.state
        {
            self.state = StateHolder::ComputedComplete {
                state: state.clone(),
                need_to_send_as_changed: *need_to_send_as_changed,
                need_to_send_as_complete: true,
            };
        }
    }

    //TODO: rename
    pub fn get_step_index(&self) -> FrameIndex {
        return self.frame_index;
    }

    pub fn is_state_deserialized(&self) -> bool {
        if let StateHolder::Deserialized { .. } = self.state {
            return true;
        } else {
            return false;
        }
    }

    pub fn is_state_none(&self) -> bool {
        if let StateHolder::None = self.state {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_changed_message(&mut self) -> Option<StateMessage<Game>> {
        if let Some((state, need_to_send_as_changed)) = match &mut self.state {
            StateHolder::None => None,
            StateHolder::Deserialized {
                state,
                need_to_send_as_changed,
            } => Some((state, need_to_send_as_changed)),
            StateHolder::ComputedIncomplete {
                state,
                need_to_send_as_changed,
            } => Some((state, need_to_send_as_changed)),
            StateHolder::ComputedComplete {
                state,
                need_to_send_as_changed,
                ..
            } => Some((state, need_to_send_as_changed)),
        } {
            if *need_to_send_as_changed {
                *need_to_send_as_changed = false;

                return Some(StateMessage::new(self.frame_index, state.clone()));
            }
        }

        return None;
    }

    pub fn get_complete_message(&mut self) -> Option<StateMessage<Game>> {
        if let StateHolder::ComputedComplete {
            state,
            need_to_send_as_complete,
            ..
        } = &mut self.state
        {
            if *need_to_send_as_complete {
                *need_to_send_as_complete = false;

                return Some(StateMessage::new(self.frame_index, state.clone()));
            }
        }
        return None;
    }
}
