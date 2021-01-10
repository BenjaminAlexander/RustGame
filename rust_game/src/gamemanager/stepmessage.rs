use crate::interface::{Input, State};
use crate::messaging::{InputMessage, StateMessage};

pub struct StepMessage<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    step_index: usize,
    inputs: Vec<Option<InputType>>,
    state: StateType
}

impl<StateType, InputType> StepMessage<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn new(step_index: usize,
           inputs: Vec<Option<InputType>>,
           state: StateType) -> Self {

        Self{step_index, inputs, state}
    }

}

impl<StateType, InputType> Clone for StepMessage<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            inputs: self.inputs.clone(),
            state: self.state.clone()
        }
    }
}