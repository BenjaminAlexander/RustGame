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

    /// The input event type.
    /// 
    /// An input event is an event that can be submitted to the client at any time 
    /// and does not correspond precisely in time to a frame.  
    type InputEvent: Send + 'static;

    /// The input aggregate type.
    /// 
    /// An aggregation of [InputEvents][Play::InputEvent] into a single input 
    /// aggregate.  An input aggregate corresponds precisely to a frame.  It represents 
    /// the complete input for a player at a particular frame.
    type InputAggregate: Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    /// The [AggregateInput] used by this game.
    type InputAggregator: AggregateInput<Game = Self>;

    /// The TCP port used by the game
    const TCP_PORT: u16;

    /// The UDP port used by the game
    const UDP_PORT: u16;

    /// The time duration between each frame of this game
    const FRAME_DURATION: TimeDuration;

    /// This duration is converted to an integer count of 
    /// [frame durations][Play::FRAME_DURATION].  That number of frame durations is 
    /// the amount of time after client inputs are due to the server that the 
    /// server will still accept an input.  
    const CLIENT_INPUT_GRACE_PERIOD: TimeDuration;

    /// This duration is converted to an integer count of 
    /// [frame durations][Play::FRAME_DURATION].  That number of frame durations 
    /// is the period between the client's ping of the server.  This ping is used 
    /// to determine the latency between the client and server and synchronize 
    /// their clocks.
    const PING_PERIOD: TimeDuration;

    /// The number of client-server clock offsets to average when synchronizing 
    /// the client's clock.
    /// 
    /// Each time the client pings the server, the client determines the offset 
    /// between its clock and the server's.  This offset is then added to a rolling 
    /// average of offsets from previous pings and the average offset is used to 
    /// synchronize the client's clock.
    const CLOCK_AVERAGE_SIZE: usize = 100;

    /// Returns the initial state of the game base on the number of players
    fn get_initial_state(player_count: usize) -> Self::State;

    /// Gets the next state of the game based on the previous [state][Play::State] 
    /// and a set of [inputs][Play::InputAggregate].
    /// 
    /// This function implements the main update logic of the game.
    fn get_next_state(arg: &UpdateArg<Self>) -> Self::State;
}
