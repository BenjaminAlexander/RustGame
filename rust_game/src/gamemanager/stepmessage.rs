use crate::interface::{Input, State, InputEvent};
use crate::messaging::{InputMessage, StateMessage};
use std::marker::PhantomData;
use std::cmp::Ordering;

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

    pub fn get_step_index(&self) -> usize {
        return self.step_index;
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

impl<StateType, InputType, InputEventType> PartialEq for StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn eq(&self, other: &Self) -> bool {
        return self.step_index.eq(&other.step_index);
    }
}

impl<StateType, InputType, InputEventType> Eq for StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

}

impl<StateType, InputType, InputEventType> PartialOrd for StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<StateType, InputType, InputEventType> Ord for StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn cmp(&self, other: &Self) -> Ordering {
        return self.step_index.cmp(&other.step_index);
    }
}