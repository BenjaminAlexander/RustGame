use crate::interface::GameTrait;
use crate::messaging::FrameIndexAndState;
use crate::FrameIndex;
use std::ops::ControlFlow;

/// A trait to observe the [FrameManager](super::frame_manager::FrameManager)
pub trait ObserveFrames: 'static + Send {
    type Game: GameTrait;

    /// Flag to indicate this oberver is the server.  This causes the
    /// [FrameManager](super::frame_manager::FrameManager) to execute server-only
    /// or client-only logic.
    const IS_SERVER: bool;

    /// Called when an [Input](super::Input) is declared authoritatively missing
    /// by the server.  This is only called on the server.
    fn input_authoritatively_missing(
        &self,
        frame_index: FrameIndex,
        player_index: usize,
    ) -> ControlFlow<()>;

    /// Called when a new State is available.  This is called both when new
    /// states are calculated and when authoritative states are inserted into
    /// the [FrameManager](super::frame_manager::FrameManager).
    fn new_state(
        &self,
        is_state_authoritative: bool,
        state_message: FrameIndexAndState<Self::Game>,
    ) -> ControlFlow<()>;
}
