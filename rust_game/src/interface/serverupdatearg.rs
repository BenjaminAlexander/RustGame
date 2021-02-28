use crate::interface::{Input, InputEvent};
use crate::messaging::InputMessage;
use std::marker::PhantomData;
use crate::gametime::TimeDuration;

#[derive(Debug)]
pub struct ServerUpdateArg<'a, InputType: Input> {
    step: usize,
    inputs: &'a Vec<Option<InputType>>,
}

impl<'a, InputType: Input> ServerUpdateArg<'a, InputType> {

    pub fn new(step: usize, inputs: &'a Vec<Option<InputType>>) -> Self {
        return Self{
            step,
            inputs,
        }
    }

    pub fn get_input(&self, player_index: usize) -> Option<&InputType> {
        if let Some(option) = self.inputs.get(player_index) {
            return option.as_ref();
        } else {
            return None;
        }
    }

    pub fn get_current_step(&self) -> usize {
        return self.step;
    }
}

impl<'a, InputType: Input> Clone for ServerUpdateArg<'a, InputType> {

    fn clone(&self) -> Self {
        return Self{
            step: self.step,
            inputs: self.inputs,
        }
    }
}