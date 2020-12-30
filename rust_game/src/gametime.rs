use crate::threading::{ChannelDrivenThread, ChannelThread, Sender, ThreadBuilder};
use timer::{Timer, Guard};
use chrono::Duration;
use std::time::Instant;
use crate::threading::sender::SendError;
use log::info;

pub struct GameTimer {
    timer: Timer,
    duration: Duration,
    start: Option<Instant>,
    guard: Option<Guard>,
}

impl GameTimer {
    pub fn new(duration: Duration) -> Self {
        GameTimer{
            timer: Timer::new(),
            duration,
            start: Option::None,
            guard: Option::None
        }
    }
}

impl ChannelDrivenThread for GameTimer {

}

impl Sender<GameTimer> {
    pub fn start(&self) -> Result<(), SendError<GameTimer>> {
        let clone = self.clone();

        self.send(|game_timer|{
            game_timer.guard = Some(game_timer.timer.schedule_repeating(game_timer.duration, move ||clone.tick()));
        })
    }

    //Called from timer thread
    fn tick(&self) {
        info!("TICK!");
        self.send(|game_timer|{
            info!("TICK!");
        }).unwrap();
    }
}