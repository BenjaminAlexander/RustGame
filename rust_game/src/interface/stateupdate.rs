use crate::interface::{State, Input, NextStateArg};

pub trait StateUpdate<StateType: State, InputType: Input>: 'static + Send {

    fn get_next_state(state: &StateType, arg: &NextStateArg<InputType>) -> StateType;

}