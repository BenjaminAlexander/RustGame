use commons::real_time::Sender;

use crate::gamemanager::{
    ManagerObserverTrait,
    StepMessage,
};
use crate::interface::{
    GameFactoryTrait,
    RenderReceiverMessage,
};
use crate::messaging::{
    ServerInputMessage,
    StateMessage,
};

pub struct ClientManagerObserver<GameFactory: GameFactoryTrait> {
    render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>,
}

impl<GameFactory: GameFactoryTrait> ClientManagerObserver<GameFactory> {
    pub fn new(
        render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>,
    ) -> Self {
        return Self {
            render_receiver_sender,
        };
    }
}

impl<GameFactory: GameFactoryTrait> ManagerObserverTrait for ClientManagerObserver<GameFactory> {
    type Factory = GameFactory::Factory;
    type Game = GameFactory::Game;

    const IS_SERVER: bool = false;

    fn on_step_message(&self, step_message: StepMessage<GameFactory::Game>) {
        let send_result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(step_message));

        //TODO: handle this without panic
        if send_result.is_err() {
            panic!("Failed to send StepMessage to render receiver");
        }
    }

    fn on_completed_step(&self, _state_message: StateMessage<GameFactory::Game>) {}

    fn on_server_input_message(&self, _server_input_message: ServerInputMessage<Self::Game>) {}
}
