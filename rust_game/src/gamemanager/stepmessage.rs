use crate::interface::{Input, State, InputEvent, NextStateArg};
use crate::messaging::{InputMessage, StateMessage};
use std::marker::PhantomData;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct StepMessage<StateType, InputType>
    where StateType: State,
          InputType: Input {

    step_index: usize,
    next_state_arg: NextStateArg<InputType>,
    state: StateType
}

impl<StateType, InputType> StepMessage<StateType, InputType>
    where StateType: State,
          InputType: Input {

    pub fn new(step_index: usize,
               next_state_arg: NextStateArg<InputType>,
               state: StateType) -> Self {

        Self{step_index, next_state_arg, state}
    }

    pub fn get_step_index(&self) -> usize {
        return self.step_index;
    }

    pub fn get_state(&self) -> &StateType {
        return &self.state;
    }
}

impl<StateType, InputType> Clone for StepMessage<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            next_state_arg: self.next_state_arg.clone(),
            state: self.state.clone()
        }
    }
}

impl<StateType, InputType> PartialEq for StepMessage<StateType, InputType>
    where StateType: State,
    InputType: Input {

    fn eq(&self, other: &Self) -> bool {
        return self.step_index.eq(&other.step_index);
    }
}

impl<StateType, InputType> Eq for StepMessage<StateType, InputType>
    where StateType: State,
          InputType: Input {

}

impl<StateType, InputType> PartialOrd for StepMessage<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<StateType, InputType> Ord for StepMessage<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn cmp(&self, other: &Self) -> Ordering {
        return self.step_index.cmp(&other.step_index);
    }
}