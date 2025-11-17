use crate::server::servercore::ServerCoreEvent;
use crate::GameTrait;
use commons::real_time::timer_service::TimerCallBack;
use commons::real_time::EventSender;

pub struct ServerGameTimerObserver<Game: GameTrait> {
    core_sender: EventSender<ServerCoreEvent<Game>>,
}

impl<Game: GameTrait> ServerGameTimerObserver<Game> {
    pub fn new(core_sender: EventSender<ServerCoreEvent<Game>>) -> Self {
        return Self { core_sender };
    }
}

impl<Game: GameTrait> TimerCallBack for ServerGameTimerObserver<Game> {
    fn tick(&mut self) {
        let send_result = self.core_sender.send_event(ServerCoreEvent::GameTimerTick);

        //TODO: handle without panic
        if send_result.is_err() {
            panic!("Failed to send GameTimerTick to Core");
        }
    }
}
