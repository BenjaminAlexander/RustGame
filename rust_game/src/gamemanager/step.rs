use crate::interface::{Input, State, InputEvent, NextStateArg};
use crate::messaging::{InputMessage, StateMessage};
use crate::gamemanager::stepmessage::StepMessage;
use std::marker::PhantomData;

pub struct Step<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    step_index: usize,
    next_state_arg: NextStateArg<InputType, InputEventType>,
    state: Option<StateType>,
    is_state_final: bool,
    is_state_complete: bool,
    need_to_compute_next_state: bool,
    need_to_send_as_complete: bool,
    need_to_send_as_changed: bool,
    phantom: PhantomData<InputEventType>
}

impl<StateType, InputType, InputEventType> Step<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn blank(step_index: usize) -> Self {

        return Self{
            step_index,
            next_state_arg: NextStateArg::new(),
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
        self.state = Some(state_message.get_state());
        self.is_state_final = true;
        self.is_state_complete = true;
        self.need_to_compute_next_state = true;
        self.need_to_send_as_complete = true;
        self.need_to_send_as_changed = true;
    }

    pub fn calculate_next_state(&mut self) -> Option<StateType> {
        if self.need_to_compute_next_state {

            let next_state = self.state.as_ref().unwrap().get_next_state(&self.next_state_arg);
            self.mark_as_calculation_not_needed();

            return Some(next_state);
        } else {
            return None;
        }
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
        self.step_index
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

    pub fn get_changed_message(&mut self) -> Option<StepMessage<StateType, InputType, InputEventType>> {
        if self.need_to_send_as_changed {
            self.need_to_send_as_changed = false;

            return Some(StepMessage::new(
                self.step_index,
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

            return Some(StateMessage::new(self.step_index, self.state.as_ref().unwrap().clone()));
        } else {
            return None;
        }

    }



}

//TODO: is this needed?
impl<StateType, InputType, InputEventType> Clone for Step<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
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

