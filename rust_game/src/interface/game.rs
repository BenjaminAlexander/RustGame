use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::interface::{State, Input, ClientUpdateArg, ServerInput, InputEvent, InterpolationResult, InterpolationArg};
use crate::interface::serverupdatearg::ServerUpdateArg;
use crate::messaging::InitialInformation;
use crate::server::Core;
use crate::threading::Sender;

pub trait Game: 'static + Send + Sized + Serialize + DeserializeOwned {
    type StateType: State;
    type InputType: Input;
    type ServerInputType: ServerInput;
    type InterpolationResultType: InterpolationResult;
    type InputEventType: InputEvent;
    type InputEventHandlerType: Send + 'static;

    fn get_server_input(state: &Self::StateType, arg: &ServerUpdateArg<Self>) -> Self::ServerInputType;

    fn get_next_state(state: &Self::StateType, arg: &ClientUpdateArg<Self>) -> Self::StateType;

    fn interpolate(initial_information: &InitialInformation<Self>,
                   first: &Self::StateType,
                   second: &Self::StateType,
                   arg: &InterpolationArg) -> Self::InterpolationResultType;

    fn new_input_event_handler() -> Self::InputEventHandlerType;

    fn handle_input_event(input_event_handler: &mut Self::InputEventHandlerType, input_event: Self::InputEventType);

    fn get_input(input_event_handler: &mut Self::InputEventHandlerType) -> Self::InputType;
}