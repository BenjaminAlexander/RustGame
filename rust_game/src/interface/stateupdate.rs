use crate::interface::{State, Input, UpdateArg, ServerInput};
use crate::interface::serverupdatearg::ServerUpdateArg;

pub trait StateUpdate<StateType: State, InputType: Input, ServerInputType: ServerInput>: 'static + Send {

    fn get_server_input(state: &StateType, arg: &ServerUpdateArg<StateType, InputType>) -> ServerInputType;

    fn get_next_state(state: &StateType, arg: &UpdateArg<StateType, InputType, ServerInputType>) -> StateType;

}