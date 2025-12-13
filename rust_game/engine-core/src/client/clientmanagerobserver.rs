use crate::frame_manager::ObserveFrames;
use crate::messaging::FrameIndexAndState;
use crate::state_channel::StateSender;
use crate::{
    FrameIndex,
    GameTrait,
};
use log::warn;
use std::ops::ControlFlow;

pub struct ClientManagerObserver<Game: GameTrait> {
    state_sender: StateSender<Game>,
}

impl<Game: GameTrait> ClientManagerObserver<Game> {
    pub fn new(state_sender: StateSender<Game>) -> Self {
        return Self { state_sender };
    }
}

impl<Game: GameTrait> ObserveFrames for ClientManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = false;

    fn new_state(
        &self,
        _is_state_authoritative: bool,
        step_message: FrameIndexAndState<Game>,
    ) -> ControlFlow<()> {
        let result = self.state_sender.send_state(step_message);

        if result.is_err() {
            warn!("Failed to send new state to render receiver");
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn input_authoritatively_missing(&self, _: FrameIndex, _: usize) -> ControlFlow<()> {
        panic!("The client should never declare inputs authoritatively missing");
    }
}
