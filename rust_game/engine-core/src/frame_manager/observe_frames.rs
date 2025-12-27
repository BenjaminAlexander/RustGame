use crate::{
    Play,
    frame_manager::{
        FrameIndexAndState,
        InputCoordinates,
    },
};
use std::ops::ControlFlow;

/// A trait to observe the [FrameManager](super::frame_manager::FrameManager)
pub trait ObserveFrames: 'static + Send {
    type Game: Play;

    /// Flag to indicate this oberver is the server.  This causes the
    /// [FrameManager](super::frame_manager::FrameManager) to execute server-only
    /// or client-only logic.
    const IS_SERVER: bool;

    /// Called when a non-authoritative [Play::InputAggregate] in the
    /// [FrameManager](super::FrameManager) is ready to be converted to an authoritative
    /// one.  
    ///
    /// On the server, this happens when the last non-authoritative input is declared
    /// authoritative because the client did not submit an authoritative input in a
    /// timely manner.
    ///
    /// On the client, this happens when the current [FrameIndex](crate::FrameIndex)
    /// is advanced and the last input must be sent to the server as authoritative.
    fn input_authoritative(
        &self,
        input_coordinates: InputCoordinates,
        input: <Self::Game as Play>::InputAggregate,
    ) -> ControlFlow<()>;

    /// Called when an [Input](super::FrameInput) is declared authoritatively missing
    /// by the server.  This is only called on the server.
    fn input_authoritatively_missing(&self, input_coordinates: InputCoordinates)
    -> ControlFlow<()>;

    /// Called when a new State is available.  This is called both when new
    /// states are calculated and when authoritative states are inserted into
    /// the [FrameManager](super::frame_manager::FrameManager).
    fn new_state(
        &self,
        is_state_authoritative: bool,
        state_message: FrameIndexAndState<Self::Game>,
    ) -> ControlFlow<()>;
}
