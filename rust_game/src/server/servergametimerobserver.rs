use crate::interface::GameFactoryTrait;
use crate::server::servercore::ServerCoreEvent;
use commons::threading::eventhandling;
use commons::time::timerservice::TimerCallBack;

pub struct ServerGameTimerObserver<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    core_sender: eventhandling::Sender<ServerCoreEvent<GameFactory::Game>>
}

impl<GameFactory: GameFactoryTrait> ServerGameTimerObserver<GameFactory> {

    pub fn new(factory: GameFactory::Factory, core_sender: eventhandling::Sender<ServerCoreEvent<GameFactory::Game>>) -> Self {
        return Self {
            factory,
            core_sender
        };
    }
}

impl<GameFactory: GameFactoryTrait> TimerCallBack for ServerGameTimerObserver<GameFactory> {
    fn tick(&mut self) {
        self.core_sender.send_event(&self.factory, ServerCoreEvent::GameTimerTick).unwrap();
    }
}
