use crate::interface::InitialInformation;
use crate::interface::InterpolationArg;
use crate::AggregateInput;
use crate::UpdateArg;
use commons::time::TimeDuration;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

//TODO: can Serialize + DeserializeOwned be removed
pub trait GameTrait: 'static + Send + Sized + Clone {
    type State: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static;

    type ClientInputEvent: Send + 'static;
    type ClientInput: Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    type InterpolationResult: Send + 'static;

    type InputAggregator: AggregateInput<
        ClientInputEvent = Self::ClientInputEvent,
        ClientInput = Self::ClientInput,
    >;

    const TCP_PORT: u16;
    const UDP_PORT: u16;
    const STEP_PERIOD: TimeDuration;
    const GRACE_PERIOD: TimeDuration;
    //TODO: rename ping period
    const PING_PERIOD: TimeDuration;
    const CLOCK_AVERAGE_SIZE: usize;

    fn get_initial_state(player_count: usize) -> Self::State;

    fn get_next_state(arg: &UpdateArg<Self>) -> Self::State;

    //TODO: this method needs to include the last interpolation result
    fn interpolate(
        initial_information: &InitialInformation<Self>,
        first: &Self::State,
        second: &Self::State,
        arg: &InterpolationArg,
    ) -> Self::InterpolationResult;
}
