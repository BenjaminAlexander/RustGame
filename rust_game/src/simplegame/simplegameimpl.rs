use crate::{SimpleInput, SimpleInputEvent, SimpleInputEventHandler, SimpleServerInput, SimpleState};
use crate::interface::{ClientUpdateArg, Game, InterpolationArg, ServerUpdateArg};
use crate::messaging::InitialInformation;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleGameImpl {

}

impl Game for SimpleGameImpl {
    type StateType = SimpleState;
    type InputType = SimpleInput;
    type ServerInputType = SimpleServerInput;
    type InterpolationResultType = SimpleState;
    type InputEventType = SimpleInputEvent;
    type InputEventHandlerType = SimpleInputEventHandler;
    
    fn get_server_input(state: &Self::StateType, arg: &ServerUpdateArg<Self>) -> Self::ServerInputType {
        return SimpleState::get_server_input(state, arg);
    }

    fn get_next_state(state: &Self::StateType, arg: &ClientUpdateArg<Self>) -> Self::StateType {
        return SimpleState::get_next_state(state, arg);
    }

    fn interpolate(initial_information: &InitialInformation<Self>, first: &Self::StateType, second: &Self::StateType, arg: &InterpolationArg) -> Self::InterpolationResultType {
        return SimpleState::interpolate(initial_information, first, second, arg);
    }

    fn new_input_event_handler() -> Self::InputEventHandlerType {
        SimpleInputEventHandler::new()
    }

    fn handle_input_event(input_event_handler: &mut Self::InputEventHandlerType, input_event: Self::InputEventType) {
        input_event_handler.handle_event(input_event);
    }

    fn get_input(input_event_handler: &mut Self::InputEventHandlerType) -> Self::InputType {
        input_event_handler.get_input()
    }
}