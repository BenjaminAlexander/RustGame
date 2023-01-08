use crate::gamemanager::{ManagerObserverTrait, RenderReceiverMessage, StepMessage};
use crate::interface::GameTrait;
use crate::messaging::{ServerInputMessage, StateMessage};
use crate::threading::ValueSender;

pub struct ClientManagerObserver<Game: GameTrait> {
    render_receiver_sender: ValueSender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> ClientManagerObserver<Game> {

    pub fn new(render_receiver_sender: ValueSender<RenderReceiverMessage<Game>>) -> Self{
        Self{render_receiver_sender}
    }

}

impl<Game: GameTrait> ManagerObserverTrait for ClientManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = false;

    fn on_step_message(&self, step_message: StepMessage<Game>) {
        self.render_receiver_sender.send(RenderReceiverMessage::StepMessage(step_message)).unwrap();
    }

    fn on_completed_step(&self, _state_message: StateMessage<Game>) {

    }

    fn on_server_input_message(&self, _server_input_message: ServerInputMessage<Self::Game>) {

    }
}