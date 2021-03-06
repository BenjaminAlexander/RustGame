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
    server_input: Option<ServerInputType>,
    inputs: Vec<Option<InputType>>,
    input_count: usize,
    is_state_final: bool,
    is_state_complete: bool,
    need_to_compute_next_state: bool,
    need_to_send_as_complete: bool,
    need_to_send_as_changed: bool,
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
            server_input: None,
            inputs: Vec::new(),
            input_count: 0,
            is_state_final: false,
            is_state_complete: false,
            need_to_compute_next_state: false,
            need_to_send_as_complete: false,
            need_to_send_as_changed: false,
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

        if self.state.is_some() {
            self.need_to_compute_next_state = true;
        }
    }

    pub fn set_server_input(&mut self, server_input: ServerInputType) {
        self.server_input = Some(server_input);

        if self.state.is_some() {
            self.need_to_compute_next_state = true;
        }
    }

    pub fn are_inputs_complete(&self) -> bool {
        return self.input_count == self.initial_information.get_player_count() &&
            self.server_input.is_some();
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

        self.state = StateHolder::Deserialized(new_state);
        self.is_state_final = true;
        self.is_state_complete = true;
        self.need_to_send_as_complete = true;

        if has_changed {
            self.need_to_compute_next_state = true;
            self.need_to_send_as_changed = true;
        }

        //info!("Set final Step: {:?}", self.step_index);
    }

    pub fn calculate_next_state(&self) -> StateHolder<StateType> {

        if let Some(state) = self.state.get_state() {

            let next_state = StateUpdateType::get_next_state(
                state,
                &self.get_update_arg()
            );

            if self.is_complete() && self.are_inputs_complete() {
                return StateHolder::ComputedComplete(next_state);
            } else {
                return StateHolder::ComputedIncomplete(next_state);
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
        self.need_to_send_as_changed = true;
        self.need_to_compute_next_state = true;
        self.state = state_holder;
    }

    pub fn mark_as_complete(&mut self) {
        self.is_state_complete = true;
        self.need_to_send_as_complete = true;
    }

    pub fn is_complete(&self) -> bool {
        self.is_state_complete
    }

    pub fn get_step_index(&self) -> usize {
        return self.step;
    }

    pub fn get_input_count(&self) -> usize {
        return self.input_count;
    }

    pub fn is_state_final(&self) -> bool {
        self.is_state_final
    }

    pub fn get_state(&self) -> Option<&StateType> {
        return self.state.get_state();
    }

    pub fn get_server_update_arg(&self) -> ServerUpdateArg<StateType, InputType> {
        return ServerUpdateArg::new(&*self.initial_information, self.step, &self.inputs);
    }

    pub fn get_update_arg(&self) -> UpdateArg<StateType, InputType, ServerInputType> {
        return UpdateArg::new(self.get_server_update_arg(), self.server_input.as_ref());
    }

    pub fn get_changed_message(&mut self) -> Option<StepMessage<StateType>> {
        if self.need_to_send_as_changed {
            self.need_to_send_as_changed = false;

            return Some(StepMessage::new(
                self.step,
                self.state.get_state().unwrap().clone()
            ));
        } else {
            return None;
        }

    }

    pub fn get_complete_message(&mut self) -> Option<StateMessage<StateType>> {
        if self.need_to_send_as_complete {
            self.need_to_send_as_complete = false;

            return Some(StateMessage::new(
                self.step,
                self.state.get_state().unwrap().clone())
            );
        } else {
            return None;
        }

    }

    pub fn get_server_input_message(&self) -> ServerInputMessage<ServerInputType> {
        return ServerInputMessage::new(
            self.step,
            self.server_input.as_ref().unwrap().clone()
        );
    }
}

pub enum StateHolder<StateType> {
    None,
    Deserialized(StateType),
    ComputedIncomplete(StateType),
    ComputedComplete(StateType)
}

impl<StateType> StateHolder<StateType> {

    pub fn get_state(&self) -> Option<&StateType> {
        return match self {
            StateHolder::None => None,
            StateHolder::Deserialized(state) => Some(state),
            StateHolder::ComputedIncomplete(state) => Some(state),
            StateHolder::ComputedComplete(state) => Some(state),
        }
    }

    pub fn is_some(&self) -> bool {
        return match self {
            StateHolder::None => false,
            _ => true,
        }
    }
}

