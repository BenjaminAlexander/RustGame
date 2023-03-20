use crate::interface::GameTrait;
use crate::server::servercore::ServerCoreEvent;
use commons::threading::eventhandling;
use commons::time::timerservice::TimerCallBack;

pub struct ServerGameTimerObserver<Game: GameTrait> {
    core_sender: eventhandling::Sender<ServerCoreEvent<Game>>
}

impl<Game: GameTrait> ServerGameTimerObserver<Game> {

    pub fn new(core_sender: eventhandling::Sender<ServerCoreEvent<Game>>) -> Self {

        Self {
            core_sender
        }

    }
}

impl<Game: GameTrait> TimerCallBack for ServerGameTimerObserver<Game> {
    fn tick(&mut self) {
        self.core_sender.send_event(ServerCoreEvent::GameTimerTick).unwrap();
    }
}
