use crate::interface::{Input, State, InputEvent};
use crate::messaging::{InputMessage, StateMessage};
use std::marker::PhantomData;

pub struct StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    step_index: usize,
    inputs: Vec<Option<InputType>>,
    state: StateType,
    phantom: PhantomData<InputEventType>
}

impl<StateType, InputType, InputEventType> StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn new(step_index: usize,
           inputs: Vec<Option<InputType>>,
           state: StateType) -> Self {

        Self{step_index, inputs, state, phantom: PhantomData}
    }

}

impl<StateType, InputType, InputEventType> Clone for StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            phantom: PhantomData
        }
    }
}