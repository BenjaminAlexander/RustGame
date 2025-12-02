use commons::real_time::Sender;

use crate::gamemanager::ManagerObserverTrait;
use crate::interface::RenderReceiverMessage;
use crate::messaging::StateMessage;
use crate::{FrameIndex, GameTrait};

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

impl<Game: GameTrait> ManagerObserverTrait for ClientManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = false;

    fn on_step_message(&self, _is_state_authoritative: bool, step_message: StateMessage<Game>) {

        let send_result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(step_message));

        //TODO: handle this without panic
        if send_result.is_err() {
            panic!("Failed to send StepMessage to render receiver");
        }
    }
    
    fn on_input_authoritatively_missing(&self, _: FrameIndex, _: usize) {
        panic!("The client should never declare inputs authoritatively missing");
    }

    
}
