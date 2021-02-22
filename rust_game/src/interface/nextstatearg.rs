use crate::interface::{Input, InputEvent};
use crate::messaging::InputMessage;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct NextStateArg<InputType: Input> {
    step: usize,
    inputs: Vec<Option<InputType>>,
    input_count: usize
}

impl<InputType: Input> NextStateArg<InputType> {

    pub fn new(step: usize) -> Self {
        return Self{
            step,
            inputs: Vec::new(),
            input_count: 0
        }
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
    }

    pub fn get_input_count(&self) -> usize {
        return self.input_count;
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

    pub fn get_next_step(&self) -> usize {
        return self.step;
    }
}

impl<InputType: Input> Clone for NextStateArg<InputType> {

    fn clone(&self) -> Self {
        return Self{
            step: self.step,
            inputs: self.inputs.clone(),
            input_count: self.input_count
        }
    }
}