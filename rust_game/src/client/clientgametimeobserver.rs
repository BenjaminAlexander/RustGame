use crate::client::ClientCoreEvent;
use crate::client::ClientCoreEvent::GameTimerTick;
use crate::interface::GameTrait;
use commons::threading::eventhandling;
use commons::time::timerservice::TimerCallBack;

pub struct ClientGameTimerObserver<Game: GameTrait> {
    core_sender: eventhandling::Sender<ClientCoreEvent<Game>>
}

impl<Game: GameTrait> ClientGameTimerObserver<Game> {

    pub fn new(core_sender: eventhandling::Sender<ClientCoreEvent<Game>>) -> Self {

        Self {
            core_sender
        }

    }
}

impl<Game: GameTrait> TimerCallBack for ClientGameTimerObserver<Game> {
    fn tick(&mut self) {
        self.core_sender.send_event(GameTimerTick).unwrap();
    }
}
