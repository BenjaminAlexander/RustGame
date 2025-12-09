use crate::frame_manager::ObserveFrames;
use crate::interface::RenderReceiverMessage;
use crate::messaging::FrameIndexAndState;
use crate::{
    FrameIndex,
    GameTrait,
};
use commons::real_time::Sender;
use log::warn;
use std::ops::ControlFlow;

pub struct ClientManagerObserver<Game: GameTrait> {
    render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
}

impl<Game: GameTrait> ClientManagerObserver<Game> {
    pub fn new(render_receiver_sender: Sender<RenderReceiverMessage<Game>>) -> Self {
        return Self {
            render_receiver_sender,
        };
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
        let result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(step_message));

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
