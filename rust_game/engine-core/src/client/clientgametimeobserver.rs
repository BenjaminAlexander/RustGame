use crate::client::ClientCoreEvent;
use crate::client::ClientCoreEvent::GameTimerTick;
use crate::interface::GameFactoryTrait;
use commons::real_time::timer_service::TimerCallBack;
use commons::real_time::EventSender;

pub struct ClientGameTimerObserver<GameFactory: GameFactoryTrait> {
    core_sender: EventSender<ClientCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> ClientGameTimerObserver<GameFactory> {
    pub fn new(core_sender: EventSender<ClientCoreEvent<GameFactory>>) -> Self {
        return Self { core_sender };
    }
}

impl<GameFactory: GameFactoryTrait> TimerCallBack for ClientGameTimerObserver<GameFactory> {
    fn tick(&mut self) {
        let send_result = self.core_sender.send_event(GameTimerTick);
        if send_result.is_err() {
            //TODO: handle this without panicing
            panic!("Failed to send GameTimerTick to the Core")
        }
    }
}
