use crate::interface::{Input, State, InputEvent, NextStateArg, StateUpdate};
use crate::messaging::{InputMessage, StateMessage};
use crate::gamemanager::stepmessage::StepMessage;
use std::marker::PhantomData;
use log::{trace, info, warn};
use crate::gametime::TimeDuration;

pub struct Step<StateType, InputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    next_state_arg: NextStateArg<InputType>,
    state: Option<StateType>,
    is_state_final: bool,
    is_state_complete: bool,
    need_to_compute_next_state: bool,
    need_to_send_as_complete: bool,
    need_to_send_as_changed: bool,
    phantom: PhantomData<StateUpdateType>,
}

impl<StateType, InputType, StateUpdateType> Step<StateType, InputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    pub fn blank(step_index: usize) -> Self {

        return Self{
            next_state_arg: NextStateArg::new(step_index),
            state: None,
            is_state_final: false,
            is_state_complete: false,
            need_to_compute_next_state: false,
            need_to_send_as_complete: false,
            need_to_send_as_changed: false,
            phantom: PhantomData
        };
    }

    pub fn set_input(&mut self, input_message: InputMessage<InputType>) {
        self.next_state_arg.set_input(input_message);

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

    pub fn calculate_next_state(&mut self) -> StateType {
        return StateUpdateType::get_next_state(self.state.as_ref().unwrap(), &self.next_state_arg);
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
        return self.next_state_arg.get_current_step();
    }

    pub fn get_input_count(&self) -> usize {
        return self.next_state_arg.get_input_count();
    }

    pub fn is_state_final(&self) -> bool {
        self.is_state_final
    }

    pub fn get_state(&self) -> Option<&StateType> {
        self.state.as_ref()
    }

    pub fn get_changed_message(&mut self) -> Option<StepMessage<StateType, InputType>> {
        if self.need_to_send_as_changed {
            self.need_to_send_as_changed = false;

            return Some(StepMessage::new(
                self.next_state_arg.get_current_step(),
                self.next_state_arg.clone(),
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
                self.next_state_arg.get_current_step(),
                self.state.as_ref().unwrap().clone())
            );
        } else {
            return None;
        }

    }
}

//TODO: is this needed?
impl<StateType, InputType, StateUpdateType> Clone for Step<StateType, InputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    fn clone(&self) -> Self {
        Self{
            next_state_arg: self.next_state_arg.clone(),
            state: self.state.clone(),
            is_state_final: self.is_state_final,
            is_state_complete: self.is_state_complete,
            need_to_compute_next_state: self.need_to_compute_next_state,
            need_to_send_as_complete: self.need_to_send_as_complete,
            need_to_send_as_changed: self.need_to_send_as_changed,
            phantom: PhantomData
        }
    }
}

