use crate::interface::{Input, State};
use crate::messaging::{InputMessage, StateMessage};
use crate::gamemanager::stepmessage::StepMessage;

pub struct Step<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    step_index: usize,
    inputs: Vec<Option<InputType>>,
    state: Option<StateType>,
    input_count: usize,
    is_state_final: bool,
    has_input_changed: bool,
    has_state_changed: bool
}

impl<StateType, InputType> Step<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn blank(step_index: usize) -> Self {

        return Self{
            step_index: step_index,
            inputs: Vec::new(),
            state: None,
            input_count: 0,
            is_state_final: false,
            has_input_changed: false,
            has_state_changed: false
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
        self.has_input_changed = true;
    }

    pub fn set_final_state(&mut self, state_message: StateMessage<StateType>) {
        self.state = Some(state_message.get_state());
        self.has_state_changed = true;
        self.is_state_final = true;
    }

    pub fn calculate_next_state(&mut self) -> Option<StateType> {
        if self.state.is_some() &&
            (self.has_state_changed || self.has_input_changed) {

            let next_state = self.state.as_ref().unwrap().get_next_state(&self.inputs);

            return Some(next_state);
        } else {
            return None;
        }
    }

    pub fn set_calculated_state(&mut self,  state: StateType) {
        self.has_state_changed = true;
        self.state = Some(state);
    }

    pub fn get_step_index(&self) -> usize {
        self.step_index
    }

    pub fn get_input_count(&self) -> usize {
        self.input_count
    }

    pub fn is_state_final(&self) -> bool {
        self.is_state_final
    }

    pub fn get_message(&self) -> StepMessage<StateType, InputType> {
        StepMessage::new(
            self.step_index,
            self.inputs.clone(),
            self.state.as_ref().unwrap().clone()
        )
    }

    pub fn get_state_message(&self) -> StateMessage<StateType> {
        StateMessage::new(self.step_index, self.state.as_ref().unwrap().clone())
    }

    pub fn get_state(&self) -> Option<&StateType> {
        self.state.as_ref()
    }

    pub fn has_state_changed(&self) -> bool {
        self.has_state_changed
    }

    pub fn mark_as_unchanged(&mut self) {
        self.has_state_changed = false;
        self.has_input_changed = false;
    }
}

//TODO: is this needed?
impl<StateType, InputType> Clone for Step<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            input_count: self.input_count,
            is_state_final: self.is_state_final,
            has_input_changed: self.has_input_changed,
            has_state_changed: self.has_state_changed
        }
    }
}

