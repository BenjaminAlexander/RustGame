use crate::interface::{Input, InputEvent, ServerUpdateArg, State};
use crate::messaging::InputMessage;
use std::marker::PhantomData;
use crate::gametime::TimeDuration;

#[derive(Debug)]
pub struct NextStateArg<'a, 'b, StateType: State, InputType: Input> {
    server_update_arg: ServerUpdateArg<'a, 'b, StateType, InputType>,
}

impl<'a, 'b, StateType: State, InputType: Input> NextStateArg<'a, 'b, StateType, InputType> {

    pub fn new(server_update_arg: ServerUpdateArg<'a, 'b, StateType, InputType>) -> Self {
        return Self{
            server_update_arg
        }
    }

    pub fn get_input(&self, player_index: usize) -> Option<&InputType> {
        return self.server_update_arg.get_input(player_index);
    }

    pub fn get_current_step(&self) -> usize {
        return self.server_update_arg.get_current_step();
    }
}