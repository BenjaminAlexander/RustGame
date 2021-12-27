use crate::interface::{ClientUpdateArg, ServerUpdateArg, GameTrait};
use crate::messaging::{InputMessage, StateMessage, ServerInputMessage, InitialInformation};
use crate::gamemanager::stepmessage::StepMessage;
use std::sync::Arc;

pub struct Step<Game: GameTrait> {

    initial_information: Arc<InitialInformation<Game>>,
    step: usize,
    state: StateHolder<Game>,
    server_input: ServerInputHolder<Game>,
    inputs: Vec<Option<Game::ClientInput>>,
    input_count: usize,
    need_to_compute_next_state: bool
}

pub enum StateHolder<Game: GameTrait> {
    None,
    Deserialized{
        state: Game::State,
        need_to_send_as_changed: bool
    },
    ComputedIncomplete{
        state: Game::State,
        need_to_send_as_changed: bool
    },
    ComputedComplete{
        state: Game::State,
        need_to_send_as_changed: bool,
        need_to_send_as_complete: bool
    }
}

pub enum ServerInputHolder<Game: GameTrait> {
    None,
    Deserialized(Game::ServerInput),
    ComputedIncomplete(Game::ServerInput),
    ComputedComplete{
        server_input: Game::ServerInput,
        need_to_send_as_complete: bool
    }
}

impl<Game: GameTrait> Step<Game> {

    pub fn blank(step_index: usize, initial_information: Arc<InitialInformation<Game>>) -> Self {

        return Self{
            initial_information,
            step: step_index,
            state: StateHolder::None,
            server_input: ServerInputHolder::None,
            inputs: Vec::new(),
            input_count: 0,
            need_to_compute_next_state: false
        };
    }

    pub fn set_input(&mut self, input_message: InputMessage<Game>) {
        let index = input_message.get_player_index();
        while self.inputs.len() <= index {
            self.inputs.push(None);
        }

        if self.inputs[index].is_none() {
            self.input_count = self.input_count + 1;
        }

        self.inputs[index] = Some(input_message.get_input());
        self.need_to_compute_next_state = true;

        if let ServerInputHolder::Deserialized(_) = self.server_input {
            //No-Op
        } else {
            self.server_input = ServerInputHolder::None;
        }
    }

    pub fn set_server_input(&mut self, server_input: Game::ServerInput) {
        self.server_input = ServerInputHolder::Deserialized(server_input);
        self.need_to_compute_next_state = true;
    }

    pub fn are_inputs_complete(&self) -> bool {

        return match self.server_input {
                ServerInputHolder::Deserialized(_) => true,
                ServerInputHolder::ComputedComplete { .. } => true,
                _ => false,
            } &&
            match self.state {
                StateHolder::Deserialized { .. } => true,
                StateHolder::ComputedComplete { .. } => true,
                _ => false,
            } &&
            self.input_count == self.initial_information.get_player_count();
    }

    pub fn set_final_state(&mut self, state_message: StateMessage<Game>) {

        let new_state = state_message.get_state();
        let mut has_changed = false;

        if let Some(old_state) = match &self.state {
            StateHolder::None => None,
            StateHolder::Deserialized{state, .. } => Some(state),
            StateHolder::ComputedIncomplete{state, .. } => Some(state),
            StateHolder::ComputedComplete{state, .. } => Some(state),
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

        self.state = StateHolder::Deserialized{
            state: new_state,
            need_to_send_as_changed: has_changed
        };

        if has_changed {
            self.need_to_compute_next_state = true;
            self.server_input = ServerInputHolder::None;
        }

        //info!("Set final Step: {:?}", self.step_index);
    }

    pub fn calculate_server_input(&mut self) {

        if let ServerInputHolder::None = self.server_input {

            if let Some(state) = match &self.state {
                StateHolder::None => None,
                StateHolder::Deserialized{state, .. } => Some(state),
                StateHolder::ComputedIncomplete{state, .. } => Some(state),
                StateHolder::ComputedComplete{state, .. } => Some(state),
            } {

                let server_input = Game::get_server_input(
                    state,
                    &self.get_server_update_arg()
                );

                if self.are_inputs_complete() {
                    self.server_input = ServerInputHolder::ComputedComplete {
                        server_input,
                        need_to_send_as_complete: true
                    };
                } else {
                    self.server_input = ServerInputHolder::ComputedIncomplete(server_input);
                }

                self.need_to_compute_next_state = true;
            }
        }
    }

    pub fn calculate_next_state(&self) -> StateHolder<Game> {

        if let Some(state) = match &self.state {
            StateHolder::None => None,
            StateHolder::Deserialized{state, .. } => Some(state),
            StateHolder::ComputedIncomplete{state, .. } => Some(state),
            StateHolder::ComputedComplete{state, .. } => Some(state),
        } {

            let server_input = match &self.server_input {
                ServerInputHolder::None => None,
                ServerInputHolder::Deserialized(server_input) => Some(server_input),
                ServerInputHolder::ComputedIncomplete(server_input) => Some(server_input),
                ServerInputHolder::ComputedComplete { server_input, .. } => Some(server_input)
            };

            let arg = ClientUpdateArg::new(
                self.get_server_update_arg(),
                server_input
            );

            let next_state = Game::get_next_state(
                state,
                &arg
            );

            if self.are_inputs_complete() {
                return StateHolder::ComputedComplete{
                    state: next_state,
                    need_to_send_as_changed: true,
                    need_to_send_as_complete: true
                };
            } else {
                return StateHolder::ComputedIncomplete{
                    state: next_state,
                    need_to_send_as_changed: true
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

    pub fn set_calculated_state(&mut self,  state_holder: StateHolder<Game>) {
        self.need_to_compute_next_state = true;
        self.state = state_holder;
    }

    pub fn mark_as_complete(&mut self) {

        if let StateHolder::ComputedIncomplete {state, need_to_send_as_changed} = &self.state {
            self.state = StateHolder::ComputedComplete {
                state: state.clone(),
                need_to_send_as_changed: *need_to_send_as_changed,
                need_to_send_as_complete: true
            };
        }

        if self.are_inputs_complete() {

            let new_server_input = match &self.server_input {
                ServerInputHolder::ComputedIncomplete(server_input) =>
                    Some(ServerInputHolder::ComputedComplete { server_input: server_input.clone(), need_to_send_as_complete: true}),
                _ => None
            };

            if let Some(server_input_holder) = new_server_input {
                self.server_input = server_input_holder;
            }
        }
    }

    pub fn get_step_index(&self) -> usize {
        return self.step;
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

    pub fn get_server_update_arg(&self) -> ServerUpdateArg<Game> {
        return ServerUpdateArg::new(&*self.initial_information, self.step, &self.inputs);
    }

    pub fn get_changed_message(&mut self) -> Option<StepMessage<Game>> {

        if let Some((state, need_to_send_as_changed)) = match &mut self.state {
            StateHolder::None => None,
            StateHolder::Deserialized{state, need_to_send_as_changed} => Some((state, need_to_send_as_changed)),
            StateHolder::ComputedIncomplete{state, need_to_send_as_changed} => Some((state, need_to_send_as_changed)),
            StateHolder::ComputedComplete{state, need_to_send_as_changed, .. } => Some((state, need_to_send_as_changed)),
        } {

            if *need_to_send_as_changed {
                *need_to_send_as_changed = false;

                return Some(StepMessage::new(
                    self.step,
                    state.clone()
                ));
            }
        }

        return None;
    }

    pub fn get_complete_message(&mut self) -> Option<StateMessage<Game>> {
        if let StateHolder::ComputedComplete{state, need_to_send_as_complete, .. } = &mut self.state {
            if *need_to_send_as_complete {
                *need_to_send_as_complete = false;

                return Some(StateMessage::new(
                    self.step,
                    state.clone()
                ));
            }
        }
        return None;
    }

    pub fn get_server_input_message(&mut self) -> Option<ServerInputMessage<Game>> {

        if let ServerInputHolder::ComputedComplete {server_input, need_to_send_as_complete} = &mut self.server_input {
            if *need_to_send_as_complete {
                *need_to_send_as_complete = false;

                return Some(ServerInputMessage::new(
                    self.step,
                    server_input.clone()
                ));
            }
        }
        return None;
    }
}
