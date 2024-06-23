use crate::interface::{EventSender, GameFactoryTrait};
use crate::server::servercore::ServerCoreEvent;
use commons::threading::eventhandling::EventSenderTrait;
use commons::time::timerservice::TimerCallBack;

pub struct ServerGameTimerObserver<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> ServerGameTimerObserver<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>,
    ) -> Self {
        return Self {
            factory,
            core_sender,
        };
    }
}

impl<GameFactory: GameFactoryTrait> TimerCallBack for ServerGameTimerObserver<GameFactory> {
    fn tick(&mut self) {
        let send_result = self.core_sender.send_event(ServerCoreEvent::GameTimerTick);

        //TODO: handle without panic
        if send_result.is_err() {
            panic!("Failed to send GameTimerTick to Core");
        }
    }
}
