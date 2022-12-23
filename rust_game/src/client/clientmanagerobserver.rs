use crate::gamemanager::{Data, ManagerObserverTrait, StepMessage};
use crate::interface::GameTrait;
use crate::messaging::{ServerInputMessage, StateMessage};
use crate::threading::ChannelDrivenThreadSender as Sender;

pub struct ClientManagerObserver<Game: GameTrait> {
    render_receiver_sender: Sender<Data<Game>>
}

impl<Game: GameTrait> ClientManagerObserver<Game> {

    pub fn new(render_receiver_sender: Sender<Data<Game>>) -> Self{
        Self{render_receiver_sender}
    }

}

impl<Game: GameTrait> ManagerObserverTrait for ClientManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = false;

    fn on_step_message(&self, step_message: StepMessage<Game>) {
        self.render_receiver_sender.on_step_message(step_message);
    }

    fn on_completed_step(&self, state_message: StateMessage<Game>) {

    }

    fn on_server_input_message(&self, server_input_message: ServerInputMessage<Self::Game>) {

    }
}