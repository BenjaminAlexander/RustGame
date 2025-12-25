use crate::{
    SimpleInput,
    SimpleInputEvent,
    SimpleInputEventHandler,
    SimpleState,
    TimeDuration,
};
use engine_core::{
    Play,
    CurrentFrame,
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
    const UDP_PORT: u16 = 3456;
    //One sec 1_000_000_000
    //60hz 16_666_666
    const FRAME_DURATION: TimeDuration = TimeDuration::new(5, 0);

    fn get_initial_state(player_count: usize) -> Self::State {
        Self::State::new(player_count)
    }

    fn next_state(arg: &CurrentFrame<Self>) -> Self::State {
        return SimpleState::get_next_state(arg);
    }
}
