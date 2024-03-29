use crate::interface::{EventSender, GameFactoryTrait};
use crate::server::servercore::ServerCoreEvent;
use commons::time::timerservice::TimerCallBack;
use commons::threading::eventhandling::EventSenderTrait;

pub struct ServerGameTimerObserver<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> ServerGameTimerObserver<GameFactory> {

    pub fn new(factory: GameFactory::Factory, core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self {
            factory,
            core_sender
        };
    }
}

impl<GameFactory: GameFactoryTrait> TimerCallBack for ServerGameTimerObserver<GameFactory> {
    fn tick(&mut self) {
        self.core_sender.send_event(ServerCoreEvent::GameTimerTick).unwrap();
    }
}
