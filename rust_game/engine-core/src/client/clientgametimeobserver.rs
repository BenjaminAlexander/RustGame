use crate::client::ClientCoreEvent;
use crate::client::ClientCoreEvent::GameTimerTick;
use crate::GameTrait;
use commons::real_time::timer_service::TimerCallBack;
use commons::real_time::EventSender;

pub struct ClientGameTimerObserver<Game: GameTrait> {
    core_sender: EventSender<ClientCoreEvent<Game>>,
}

impl<Game: GameTrait> ClientGameTimerObserver<Game> {
    pub fn new(core_sender: EventSender<ClientCoreEvent<Game>>) -> Self {
        return Self { core_sender };
    }
}

impl<Game: GameTrait> TimerCallBack for ClientGameTimerObserver<Game> {
    fn tick(&mut self) {
        let send_result = self.core_sender.send_event(GameTimerTick);
        if send_result.is_err() {
            //TODO: handle this without panicing
            panic!("Failed to send GameTimerTick to the Core")
        }
    }
}
