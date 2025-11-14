use crate::interface::GameFactoryTrait;
use crate::server::servercore::ServerCoreEvent;
use commons::real_time::EventSender;
use commons::time::timerservice::TimerCallBack;

pub struct ServerGameTimerObserver<GameFactory: GameFactoryTrait> {
    core_sender: EventSender<ServerCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> ServerGameTimerObserver<GameFactory> {
    pub fn new(core_sender: EventSender<ServerCoreEvent<GameFactory>>) -> Self {
        return Self { core_sender };
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
