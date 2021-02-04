use crate::interface::{Input, State, InputEvent, NextStateArg};
use crate::messaging::{InputMessage, StateMessage};
use std::marker::PhantomData;
use std::cmp::Ordering;

pub struct StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    step_index: usize,
    next_state_arg: NextStateArg<InputType, InputEventType>,
    state: StateType,
    phantom: PhantomData<InputEventType>
}

impl<StateType, InputType, InputEventType> StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn new(step_index: usize,
               next_state_arg: NextStateArg<InputType, InputEventType>,
           state: StateType) -> Self {

        Self{step_index, next_state_arg, state, phantom: PhantomData}
    }

    pub fn get_step_index(&self) -> usize {
        return self.step_index;
    }

    pub fn get_state(&self) -> &StateType {
        return &self.state;
    }
}

impl<StateType, InputType, InputEventType> Clone for StepMessage<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            next_state_arg: self.next_state_arg.clone(),
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