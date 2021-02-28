use crate::interface::{Input, State, InputEvent, NextStateArg};
use crate::messaging::{InputMessage, StateMessage};
use std::marker::PhantomData;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct StepMessage<StateType>
    where StateType: State {

    step_index: usize,
    state: StateType
}

impl<StateType> StepMessage<StateType>
    where StateType: State {

    pub fn new(step_index: usize,
               state: StateType) -> Self {

        Self{step_index, state}
    }

    pub fn get_step_index(&self) -> usize {
        return self.step_index;
    }

    pub fn get_state(&self) -> &StateType {
        return &self.state;
    }
}

impl<StateType> Clone for StepMessage<StateType>
    where StateType: State {

    fn clone(&self) -> Self {
        Self{
            step_index: self.step_index,
            state: self.state.clone()
        }
    }
}

impl<StateType> PartialEq for StepMessage<StateType>
    where StateType: State {

    fn eq(&self, other: &Self) -> bool {
        return self.step_index.eq(&other.step_index);
    }
}

impl<StateType> Eq for StepMessage<StateType>
    where StateType: State {

}

impl<StateType> PartialOrd for StepMessage<StateType>
    where StateType: State {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<StateType> Ord for StepMessage<StateType>
    where StateType: State {

    fn cmp(&self, other: &Self) -> Ordering {
        return self.step_index.cmp(&other.step_index);
    }
}