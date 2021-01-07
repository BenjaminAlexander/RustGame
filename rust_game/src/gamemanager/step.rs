use crate::interface::{Input, State};
use crate::messaging::{InputMessage, StateMessage};

pub struct Step<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    step_index: usize,
    inputs: Vec<Option<InputType>>,
    state: Option<StateType>,
    is_state_final: bool,
    has_changed_since_last_calculation: bool
}

impl<StateType, InputType> Step<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn blank(sequence: usize, player_count: usize) -> Self {
        let mut inputs = Vec::<Option<InputType>>::with_capacity(player_count);
        for i in 0..player_count {
            inputs[i] = None;
        }
        return Self{
            step_index: sequence,
            inputs,
            state: None,
            is_state_final: false,
            has_changed_since_last_calculation: false
        };
    }

    pub fn set_input(&mut self, input_message: InputMessage<InputType>) {
        let index = input_message.get_player_index();
        self.inputs[index] = Some(input_message.get_input());
        self.has_changed_since_last_calculation = true;
    }

    pub fn set_final_state(&mut self, state_message: StateMessage<StateType>) {
        self.state = Some(state_message.get_state());
        self.has_changed_since_last_calculation = true;
        self.is_state_final = true;
    }

    pub fn update_next(&mut self,  next: &mut Self) -> bool {
        if self.state.is_some() &&
            self.has_changed_since_last_calculation &&
            !next.is_state_final {

            self.has_changed_since_last_calculation = false;
            next.has_changed_since_last_calculation = true;

            next.state = Some(self.state.as_ref().unwrap().get_next_state(&self.inputs));
            
            return true;
        } else {
            return false;
        }
    }

    pub fn get_step_index(&self) -> usize {
        self.step_index
    }

}

impl<StateType, InputType> Clone for Step<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            is_state_final: self.is_state_final,
            has_changed_since_last_calculation: self.has_changed_since_last_calculation
        }
    }
}

