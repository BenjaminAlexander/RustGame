use crate::{SimpleInput, SimpleInputEvent, SimpleInputEventHandler, SimpleServerInput, SimpleState, TimeDuration};
use engine_core::interface::{ClientUpdateArg, GameTrait, InterpolationArg, ServerUpdateArg};
use engine_core::messaging::InitialInformation;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleGameImpl {

}

impl GameTrait for SimpleGameImpl {
    type State = SimpleState;
    type ClientInput = SimpleInput;
    type ServerInput = SimpleServerInput;
    type InterpolationResult = SimpleState;
    type ClientInputEvent = SimpleInputEvent;
    type ClientInputEventHandler = SimpleInputEventHandler;

    const TCP_PORT: u16 = 3456;
    const UDP_PORT: u16 = 3457;
    const STEP_PERIOD: TimeDuration = TimeDuration::from_seconds(0.1);
    const GRACE_PERIOD: TimeDuration = TimeDuration::from_seconds(1.0);
    const TIME_SYNC_MESSAGE_PERIOD: TimeDuration = TimeDuration::from_seconds(1.0);
    const CLOCK_AVERAGE_SIZE: usize = 100;

    fn get_initial_state(player_count: usize) -> Self::State {
        Self::State::new(player_count)
    }

    fn get_server_input(state: &Self::State, arg: &ServerUpdateArg<Self>) -> Self::ServerInput {
        return SimpleState::get_server_input(state, arg);
    }

    fn get_next_state(state: &Self::State, arg: &ClientUpdateArg<Self>) -> Self::State {
        return SimpleState::get_next_state(state, arg);
    }

    fn interpolate(initial_information: &InitialInformation<Self>, first: &Self::State, second: &Self::State, arg: &InterpolationArg) -> Self::InterpolationResult {
        return SimpleState::interpolate(initial_information, first, second, arg);
    }

    fn new_input_event_handler() -> Self::ClientInputEventHandler {
        SimpleInputEventHandler::new()
    }

    fn handle_input_event(input_event_handler: &mut Self::ClientInputEventHandler, input_event: Self::ClientInputEvent) {
        input_event_handler.handle_event(input_event);
    }

    fn get_input(input_event_handler: &mut Self::ClientInputEventHandler) -> Self::ClientInput {
        input_event_handler.get_input()
    }
}