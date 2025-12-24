use crate::AggregateInput;
use crate::UpdateArg;
use commons::time::TimeDuration;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// Defines the logic of a game
pub trait Play: 'static + Send + Clone {

    /// A state of the game at a frame.
    type State: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static;

    /// An input event that can be sent to the local client at any time.
    type InputEvent: Send + 'static;

    /// An aggregation of [input events][Play::InputEvent] into a single input 
    /// aggregate.  This represents the complete input for a player at a frame.
    type InputAggregate: Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    /// The [AggregateInput] used by this game.
    type InputAggregator: AggregateInput<Game = Self>;

    const TCP_PORT: u16;
    const UDP_PORT: u16;
    const STEP_PERIOD: TimeDuration;
    const GRACE_PERIOD: TimeDuration;
    const PING_PERIOD: TimeDuration;
    const CLOCK_AVERAGE_SIZE: usize;

    fn get_initial_state(player_count: usize) -> Self::State;

    fn get_next_state(arg: &UpdateArg<Self>) -> Self::State;
}
