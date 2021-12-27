use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::interface::{ClientUpdateArg, InterpolationArg};
use crate::interface::serverupdatearg::ServerUpdateArg;
use crate::messaging::InitialInformation;
use crate::TimeDuration;

//TODO: can Serialize + DeserializeOwned be removed
pub trait GameTrait: 'static + Send + Sized {

    type State:
        Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static;

    type ClientInput:
        Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    type ServerInput:
        Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    type InterpolationResult:
        Send + 'static;

    type ClientInputEvent:
        Send + 'static;

    type ClientInputEventHandler:
        Send + 'static;

    const TCP_PORT: u16;
    const UDP_PORT: u16;
    const STEP_PERIOD: TimeDuration;
    const GRACE_PERIOD: TimeDuration;
    const TIME_SYNC_MESSAGE_PERIOD: TimeDuration;
    const CLOCK_AVERAGE_SIZE: usize;

    fn get_initial_state(player_count: usize) -> Self::State;

    fn get_server_input(state: &Self::State, arg: &ServerUpdateArg<Self>) -> Self::ServerInput;

    fn get_next_state(state: &Self::State, arg: &ClientUpdateArg<Self>) -> Self::State;

    fn interpolate(initial_information: &InitialInformation<Self>,
                   first: &Self::State,
                   second: &Self::State,
                   arg: &InterpolationArg) -> Self::InterpolationResult;

    fn new_input_event_handler() -> Self::ClientInputEventHandler;

    fn handle_input_event(input_event_handler: &mut Self::ClientInputEventHandler, input_event: Self::ClientInputEvent);

    fn get_input(input_event_handler: &mut Self::ClientInputEventHandler) -> Self::ClientInput;
}