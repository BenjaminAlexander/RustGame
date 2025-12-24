use crate::{
    SimpleInput,
    SimpleInputEvent,
    SimpleInputEventHandler,
    SimpleState,
    TimeDuration,
};
use engine_core::{
    Play,
    UpdateArg,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleGameImpl {}

impl Play for SimpleGameImpl {
    type State = SimpleState;
    type InputAggregate = SimpleInput;
    type InputEvent = SimpleInputEvent;
    type InputAggregator = SimpleInputEventHandler;

    const TCP_PORT: u16 = 3456;
    const UDP_PORT: u16 = 3457;
    //One sec 1_000_000_000
    //60hz 16_666_666
    const FRAME_DURATION: TimeDuration = TimeDuration::new(5, 0);
    const CLIENT_INPUT_GRACE_PERIOD: TimeDuration = TimeDuration::new(1, 0);
    const PING_PERIOD: TimeDuration = TimeDuration::new(1, 0);

    fn get_initial_state(player_count: usize) -> Self::State {
        Self::State::new(player_count)
    }

    fn get_next_state(arg: &UpdateArg<Self>) -> Self::State {
        return SimpleState::get_next_state(arg);
    }
}
