use crate::interface::{Input, State, InputEvent, NextStateArg, StateUpdate, ServerInput, ServerUpdateArg};
use crate::messaging::{InputMessage, StateMessage, ServerInputMessage};
use crate::gamemanager::stepmessage::StepMessage;
use std::marker::PhantomData;
use log::{trace, info, warn};
use crate::gametime::TimeDuration;

pub struct Step<StateType, InputType, ServerInputType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput {

    step: usize,
    state: Option<StateType>,
    server_input: Option<ServerInputType>,
    inputs: Vec<Option<InputType>>,
    input_count: usize,
    is_state_final: bool,
    is_state_complete: bool,
    need_to_compute_next_state: bool,
    need_to_send_as_complete: bool,
    need_to_send_as_changed: bool,
}

impl<StateType, InputType, ServerInputType> Step<StateType, InputType, ServerInputType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput {

    pub fn blank(step_index: usize) -> Self {

        return Self{
            step: step_index,
            state: None,
            server_input: None,
            inputs: Vec::new(),
            input_count: 0,
            is_state_final: false,
            is_state_complete: false,
            need_to_compute_next_state: false,
            need_to_send_as_complete: false,
            need_to_send_as_changed: false
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

    pub fn set_final_state(&mut self, state_message: StateMessage<StateType>) {

        let new_state = state_message.get_state();
        let mut has_changed = false;

        if let Some(old_state) = &self.state {
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

        self.state = Some(new_state);
        self.is_state_final = true;
        self.is_state_complete = true;
        self.need_to_send_as_complete = true;

        if has_changed {
            self.need_to_compute_next_state = true;
            self.need_to_send_as_changed = true;
        }

        //info!("Set final Step: {:?}", self.step_index);
    }

    pub fn need_to_compute_next_state(&self) -> bool {
        return self.need_to_compute_next_state;
    }

    pub fn mark_as_calculation_not_needed(&mut self) {
        self.need_to_compute_next_state = false;
    }

    pub fn set_calculated_state(&mut self,  state: StateType) {
        self.need_to_send_as_changed = true;
        self.need_to_compute_next_state = true;
        self.state = Some(state);
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
        self.state.as_ref()
    }

    pub fn get_server_update_arg(&self) -> ServerUpdateArg<InputType> {
        return ServerUpdateArg::new(self.step, &self.inputs);
    }

    pub fn get_update_arg(&self) -> NextStateArg<InputType> {
        return NextStateArg::new(self.step, &self.inputs);
    }

    pub fn get_changed_message(&mut self) -> Option<StepMessage<StateType>> {
        if self.need_to_send_as_changed {
            self.need_to_send_as_changed = false;

            return Some(StepMessage::new(
                self.step,
                self.state.as_ref().unwrap().clone()
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
                self.state.as_ref().unwrap().clone())
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

//TODO: is this needed?
impl<StateType, InputType, ServerInputType> Clone for Step<StateType, InputType, ServerInputType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput {

    fn clone(&self) -> Self {
        Self {
            step: self.step,
            state: self.state.clone(),
            server_input: self.server_input.clone(),
            inputs: self.inputs.clone(),
            input_count: self.input_count,
            is_state_final: self.is_state_final,
            is_state_complete: self.is_state_complete,
            need_to_compute_next_state: self.need_to_compute_next_state,
            need_to_send_as_complete: self.need_to_send_as_complete,
            need_to_send_as_changed: self.need_to_send_as_changed
        }
    }
}

