use crate::{
    SimpleInput,
    SimpleInputEvent,
    SimpleInputEventHandler,
    SimpleState,
    TimeDuration,
};
use engine_core::{
    GameTrait,
    InitialInformation,
    InterpolationArg,
    UpdateArg,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleGameImpl {}

impl GameTrait for SimpleGameImpl {
    type State = SimpleState;
    type ClientInput = SimpleInput;
    type InterpolationResult = SimpleState;
    type ClientInputEvent = SimpleInputEvent;
    type InputAggregator = SimpleInputEventHandler;

    const TCP_PORT: u16 = 3456;
    const UDP_PORT: u16 = 3457;
    const STEP_PERIOD: TimeDuration = TimeDuration::new(0, 1000_000_000);
    const GRACE_PERIOD: TimeDuration = TimeDuration::new(1, 0);
    const PING_PERIOD: TimeDuration = TimeDuration::new(1, 0);
    const CLOCK_AVERAGE_SIZE: usize = 100;

    fn get_initial_state(player_count: usize) -> Self::State {
        Self::State::new(player_count)
    }

    fn get_next_state(arg: &UpdateArg<Self>) -> Self::State {
        return SimpleState::get_next_state(arg);
    }

    fn interpolate(
        initial_information: &InitialInformation<Self>,
        first: &Self::State,
        second: &Self::State,
        arg: &InterpolationArg,
    ) -> Self::InterpolationResult {
        return SimpleState::interpolate(initial_information, first, second, arg);
    }
}
