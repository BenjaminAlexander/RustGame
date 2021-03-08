use crate::interface::{Input, State, InputEvent, UpdateArg, StateUpdate, ServerInput, ServerUpdateArg};
use crate::messaging::{InputMessage, StateMessage, ServerInputMessage, InitialInformation};
use crate::gamemanager::stepmessage::StepMessage;
use std::marker::PhantomData;
use log::{trace, info, warn};
use crate::gametime::TimeDuration;
use std::sync::Arc;

pub struct Step<StateType, InputType, ServerInputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    initial_information: Arc<InitialInformation<StateType>>,
    step: usize,
    state: StateHolder<StateType>,
    server_input: ServerInputHolder<ServerInputType>,
    inputs: Vec<Option<InputType>>,
    input_count: usize,
    need_to_compute_next_state: bool,
    phantom: PhantomData<StateUpdateType>
}

impl<StateType, InputType, ServerInputType, StateUpdateType> Step<StateType, InputType, ServerInputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    pub fn blank(step_index: usize, initial_information: Arc<InitialInformation<StateType>>) -> Self {

        return Self{
            initial_information,
            step: step_index,
            state: StateHolder::None,
            server_input: ServerInputHolder::None,
            inputs: Vec::new(),
            input_count: 0,
            need_to_compute_next_state: false,
            phantom: PhantomData
        };
    }

    pub fn set_input(&mut self, input_message: InputMessage<InputType>) {
        let index = input_message.get_player_index();
        while self.inputs.len() <= index {
            self.inputs.push(None);
        }

        if self.inputs[index].is_none() {
            self.input_count = self.input_count + 1;
        }

        self.inputs[index] = Some(input_message.get_input());
        self.server_input = ServerInputHolder::None;

        if self.state.is_some() {
            self.need_to_compute_next_state = true;
        }
    }

    pub fn set_server_input(&mut self, server_input: ServerInputType) {
        self.server_input = ServerInputHolder::Deserialized(server_input);

        if self.state.is_some() {
            self.need_to_compute_next_state = true;
        }
    }

    pub fn are_inputs_complete(&self) -> bool {
        return self.input_count == self.initial_information.get_player_count() &&
            self.server_input.is_complete() &&
            self.state.is_complete();
    }

    pub fn set_final_state(&mut self, state_message: StateMessage<StateType>) {

        let new_state = state_message.get_state();
        let mut has_changed = false;

        if let Some(old_state) = self.state.get_state() {
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
            if let Some(state) = self.get_state() {
                let server_input = StateUpdateType::get_server_input(
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

    pub fn calculate_next_state(&self) -> StateHolder<StateType> {

        if let Some(state) = self.state.get_state() {

            let arg = UpdateArg::new(
                self.get_server_update_arg(),
                self.server_input.get_server_input()
            );

            let next_state = StateUpdateType::get_next_state(
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

    pub fn set_calculated_state(&mut self,  state_holder: StateHolder<StateType>) {
        self.need_to_compute_next_state = true;
        self.state = state_holder;
    }

    pub fn mark_as_complete(&mut self) {
        if let Some(state_holder) = self.state.mark_as_complete() {
            self.state = state_holder;
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

    pub fn get_state(&self) -> Option<&StateType> {
        return self.state.get_state();
    }

    pub fn get_server_update_arg(&self) -> ServerUpdateArg<StateType, InputType> {
        return ServerUpdateArg::new(&*self.initial_information, self.step, &self.inputs);
    }

    pub fn get_changed_message(&mut self) -> Option<StepMessage<StateType>> {
        if let Some(state) = self.state.get_changed_state() {
            return Some(StepMessage::new(
                self.step,
                state.clone()
            ));
        } else {
            return None;
        }
    }

    pub fn get_complete_message(&mut self) -> Option<StateMessage<StateType>> {
        if let Some(state) = self.state.get_complete_state() {
            return Some(StateMessage::new(
                self.step,
                state.clone()
            ));
        } else {
            return None;
        }
    }

    pub fn get_server_input_message(&mut self) -> Option<ServerInputMessage<ServerInputType>> {

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

#[derive(Clone)]
pub enum StateHolder<StateType: State> {
    None,
    Deserialized{
        state: StateType,
        need_to_send_as_changed: bool
    },
    ComputedIncomplete{
        state: StateType,
        need_to_send_as_changed: bool
    },
    ComputedComplete{
        state: StateType,
        need_to_send_as_changed: bool,
        need_to_send_as_complete: bool
    }
}

impl<StateType: State> StateHolder<StateType> {

    pub fn is_complete(&self) -> bool {
        return match self {
            StateHolder::Deserialized { .. } => true,
            StateHolder::ComputedComplete { .. } => true,
            _ => false,
        };
    }

    pub fn get_state(&self) -> Option<&StateType> {
        return match self {
            StateHolder::None => None,
            StateHolder::Deserialized{state, .. } => Some(state),
            StateHolder::ComputedIncomplete{state, .. } => Some(state),
            StateHolder::ComputedComplete{state, .. } => Some(state),
        }
    }

    pub fn mark_as_complete(&self) -> Option<Self> {
        return match self {
            StateHolder::ComputedIncomplete {state, need_to_send_as_changed} =>
                Some(StateHolder::ComputedComplete {
                    state: state.clone(),
                    need_to_send_as_changed: *need_to_send_as_changed,
                    need_to_send_as_complete: true
                }),
            _ => None,
        }
    }

    pub fn is_some(&self) -> bool {
        return match self {
            StateHolder::None => false,
            _ => true,
        }
    }

    pub fn get_changed_state(&mut self) -> Option<&StateType> {

        if let StateHolder::Deserialized{state, need_to_send_as_changed} = self {
            if *need_to_send_as_changed {
                *need_to_send_as_changed = false;
                return Some(state);
            }
        } else if let StateHolder::ComputedIncomplete{state, need_to_send_as_changed} = self {
            if *need_to_send_as_changed {
                *need_to_send_as_changed = false;
                return Some(state);
            }
        } else if let StateHolder::ComputedComplete{state, need_to_send_as_changed, .. } = self {
            if *need_to_send_as_changed {
                *need_to_send_as_changed = false;
                return Some(state);
            }
        }
        return None;
    }

    pub fn get_complete_state(&mut self) -> Option<&StateType> {
        if let StateHolder::ComputedComplete{state, need_to_send_as_complete, .. } = self {
            if *need_to_send_as_complete {
                *need_to_send_as_complete = false;
                return Some(state);
            }
        }
        return None;
    }
}

pub enum ServerInputHolder<ServerInputType: ServerInput> {
    None,
    Deserialized(ServerInputType),
    ComputedIncomplete(ServerInputType),
    ComputedComplete{
        server_input: ServerInputType,
        need_to_send_as_complete: bool
    }
}

impl<ServerInputType: ServerInput> ServerInputHolder<ServerInputType> {
    pub fn is_complete(&self) -> bool {
        return match self {
            ServerInputHolder::Deserialized(_) => true,
            ServerInputHolder::ComputedComplete { .. } => true,
            _ => false,
        };
    }

    fn get_server_input(&self) -> Option<&ServerInputType> {
        match self {
            ServerInputHolder::None => None,
            ServerInputHolder::Deserialized(server_input) => Some(server_input),
            ServerInputHolder::ComputedIncomplete(server_input) => Some(server_input),
            ServerInputHolder::ComputedComplete { server_input, .. } => Some(server_input)
        }
    }
}
