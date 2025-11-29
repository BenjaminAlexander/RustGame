use commons::real_time::Factory;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{FrameIndex, GameTrait, game_time::{
    FrameDuration,
    StartTime,
}};

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ServerConfig {
    start_time: StartTime,
    frame_duration: FrameDuration,
    input_grace_period_frames: usize,
}

impl ServerConfig {
    pub fn new<Game: GameTrait>(factory: &Factory) -> Self {
        let now = factory.get_time_source().now();
        let frame_duration = FrameDuration::new(Game::STEP_PERIOD);
        let input_grace_period_frames = frame_duration.to_frame_count(&Game::GRACE_PERIOD) as usize;

        return Self {
            start_time: StartTime::new(now),
            frame_duration,
            input_grace_period_frames
        };
    }

    pub fn get_frame_duration(&self) -> &FrameDuration {
        &self.frame_duration
    }

    pub fn get_start_time(&self) -> &StartTime {
        &self.start_time
    }

    pub fn get_input_grace_period_frames(&self) -> usize {
        self.input_grace_period_frames
    }

    /// Returns the last frame index which is still open for clients to submit inputs
    pub fn get_last_open_frame_index(&self, current_frame_index: FrameIndex) -> FrameIndex {
        if current_frame_index.usize() > self.input_grace_period_frames {
            current_frame_index - self.input_grace_period_frames
        } else {
            FrameIndex::zero()
        }
    }
}
