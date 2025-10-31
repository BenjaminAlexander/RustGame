use commons::time::TimeDuration;
use serde::{
    Deserialize,
    Serialize,
};

/// The configuration values for the game timer
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct GameTimerConfig {

    /// The [`TimeDuration`] between frames
    frame_duration: TimeDuration,
}

impl GameTimerConfig {
    pub fn new(frame_duration: TimeDuration) -> Self {
        return Self { frame_duration };
    }

    pub fn get_frame_duration(&self) -> &TimeDuration {
        return &self.frame_duration;
    }

    //TODO: get FrameIndex
}
