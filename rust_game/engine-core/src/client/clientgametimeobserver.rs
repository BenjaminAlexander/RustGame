use crate::Play;
use crate::client::ClientCoreEvent;
use crate::client::ClientCoreEvent::GameTimerTick;
use commons::real_time::EventSender;
use commons::real_time::timer_service::TimerCallBack;

pub struct ClientGameTimerObserver<Game: Play> {
    core_sender: EventSender<ClientCoreEvent<Game>>,
}

impl<Game: Play> ClientGameTimerObserver<Game> {
    pub fn new(core_sender: EventSender<ClientCoreEvent<Game>>) -> Self {
        return Self { core_sender };
    }
}

impl<Game: Play> TimerCallBack for ClientGameTimerObserver<Game> {
    fn tick(&mut self) {
        let send_result = self.core_sender.send_event(GameTimerTick);
        if send_result.is_err() {
            //TODO: handle this without panicing
            panic!("Failed to send GameTimerTick to the Core")
        }
    }
}
