use crate::interface::{Input, InputEvent, ServerUpdateArg, State, ServerInput};
use crate::messaging::InputMessage;
use std::marker::PhantomData;
use crate::gametime::TimeDuration;

#[derive(Debug)]
pub struct UpdateArg<'a, 'b, 'c, StateType: State, InputType: Input, ServerInputType: ServerInput> {
    server_update_arg: ServerUpdateArg<'a, 'b, StateType, InputType>,
    server_input: Option<&'c ServerInputType>
}

impl<'a, 'b, 'c, StateType: State, InputType: Input, ServerInputType: ServerInput> UpdateArg<'a, 'b, 'c, StateType, InputType, ServerInputType> {

    pub fn new(server_update_arg: ServerUpdateArg<'a, 'b, StateType, InputType>,
               server_input: Option<&'c ServerInputType>) -> Self {
        return Self{
            server_update_arg,
            server_input
        }
    }

    pub fn get_input(&self, player_index: usize) -> Option<&InputType> {
        return self.server_update_arg.get_input(player_index);
    }

    pub fn get_current_step(&self) -> usize {
        return self.server_update_arg.get_current_step();
    }

    pub fn get_server_input(&self) -> Option<&ServerInputType> {
        return self.server_input;
    }
}