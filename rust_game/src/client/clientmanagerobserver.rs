use crate::gamemanager::{ManagerObserverTrait, RenderReceiverMessage, StepMessage};
use crate::interface::GameFactoryTrait;
use crate::messaging::{ServerInputMessage, StateMessage};
use commons::threading::channel::Sender;

pub struct ClientManagerObserver<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>
}

impl<GameFactory: GameFactoryTrait> ClientManagerObserver<GameFactory> {

    pub fn new(factory: GameFactory::Factory, render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>) -> Self{
        return Self {
            factory,
            render_receiver_sender
        };
    }

}

impl<GameFactory: GameFactoryTrait> ManagerObserverTrait for ClientManagerObserver<GameFactory> {
    type Factory = GameFactory::Factory;
    type Game = GameFactory::Game;

    const IS_SERVER: bool = false;

    fn on_step_message(&self, step_message: StepMessage<GameFactory::Game>) {
        self.render_receiver_sender.send(&self.factory, RenderReceiverMessage::StepMessage(step_message)).unwrap();
    }

    fn on_completed_step(&self, _state_message: StateMessage<GameFactory::Game>) {

    }

    fn on_server_input_message(&self, _server_input_message: ServerInputMessage<Self::Game>) {

    }
}