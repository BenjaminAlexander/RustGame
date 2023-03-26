use crate::client::ClientCoreEvent;
use crate::client::ClientCoreEvent::GameTimerTick;
use crate::interface::GameFactoryTrait;
use commons::threading::eventhandling;
use commons::threading::eventhandling::EventSenderTrait;
use commons::time::timerservice::TimerCallBack;

pub struct ClientGameTimerObserver<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    core_sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> ClientGameTimerObserver<GameFactory> {

    pub fn new(factory: GameFactory::Factory, core_sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>) -> Self {
        return Self {
            factory,
            core_sender
        };
    }
}

impl<GameFactory: GameFactoryTrait> TimerCallBack for ClientGameTimerObserver<GameFactory> {
    fn tick(&mut self) {
        self.core_sender.send_event(GameTimerTick).unwrap();
    }
}
