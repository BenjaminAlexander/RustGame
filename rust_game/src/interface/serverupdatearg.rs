use crate::interface::{Input, InputEvent, State};
use crate::messaging::{InputMessage, InitialInformation};
use std::marker::PhantomData;
use crate::gametime::TimeDuration;
use crate::interface::game::Game;

#[derive(Debug)]
pub struct ServerUpdateArg<'a, 'b, GameType: Game> {
    initial_information: &'a InitialInformation<GameType>,
    step: usize,
    inputs: &'b Vec<Option<GameType::InputType>>,
}

impl<'a, 'b, GameType: Game> ServerUpdateArg<'a, 'b, GameType> {

    pub fn new(initial_information: &'a InitialInformation<GameType>, step: usize, inputs: &'b Vec<Option<GameType::InputType>>) -> Self {
        return Self{
            initial_information,
            step,
            inputs,
        }
    }

    pub fn get_input(&self, player_index: usize) -> Option<&GameType::InputType> {
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
        return self.get_current_step() + 1;
    }

    pub fn get_current_duration_since_start(&self) -> TimeDuration {
        return self.initial_information.get_server_config().get_step_duration() * self.step as i64;
    }

    pub fn get_next_step_duration_since_start(&self) -> TimeDuration {
        return self.initial_information.get_server_config().get_step_duration() * self.get_next_step() as i64;
    }
}