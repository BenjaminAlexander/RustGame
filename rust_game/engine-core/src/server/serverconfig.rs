use serde::{
    Deserialize,
    Serialize,
};

use crate::game_time::{
    FrameDuration,
    StartTime,
};

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ServerConfig {
    start_time: StartTime,
    frame_duration: FrameDuration,
}

impl ServerConfig {
    pub fn new(start_time: StartTime, frame_duration: FrameDuration) -> Self {
        return Self {
            start_time,
            frame_duration,
        };
    }

    pub fn get_frame_duration(&self) -> &FrameDuration {
        return &self.frame_duration;
    }

    pub fn get_start_time(&self) -> &StartTime {
        return &self.start_time;
    }
}
