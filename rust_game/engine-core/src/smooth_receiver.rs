use commons::time::TimeValue;
use log::warn;

use crate::{
    InitialInformation,
    StateReceiver,
    StateReceiverError,
    game_time::StartTime,
    smooth::{
        Smooth,
        SmoothArguments,
    },
};

/// A [SmoothReceiver] uses a [StateReceiver] and a [Smooth] implementation to
/// perform smoothing of corrections and updates to the game's state.
pub struct SmoothReceiver<T: Smooth> {
    state_receiver: StateReceiver<T::Game>,
    last_result: Option<SmoothedStateAndTimeValue<T>>,
}

struct SmoothedStateAndTimeValue<T: Smooth> {
    smoothed_state: T::SmoothedState,
    time_value: TimeValue,
}

/// A struct containing the smoothed result
pub struct SmoothedState<'a, T: Smooth> {
    pub start_time: &'a StartTime,
    pub initial_information: &'a InitialInformation<T::Game>,

    /// The time of the [SmoothedState::current_state]
    pub current_time: &'a TimeValue,

    /// The most recent smoothed state
    pub current_state: &'a T::SmoothedState,
}

impl<T: Smooth> SmoothReceiver<T> {
    /// Creates a new Smooth Receiver
    pub fn new(state_receiver: StateReceiver<T::Game>) -> Self {
        Self {
            state_receiver,
            last_result: None,
        }
    }

    /// Gets the most recent smoothed state if it is available
    pub fn get_smoothed_state(&mut self) -> Result<SmoothedState<'_, T>, StateReceiverError> {
        let current_states = self.state_receiver.get_most_recent_state()?;

        let now = current_states.get_time_source().now();

        let result = match &self.last_result {
            Some(last_result_and_time) => {
                let future_frame_time = current_states.get_start_time().get_frame_time_of_occurence(
                    current_states
                        .get_initial_information()
                        .get_server_config()
                        .get_frame_duration(),
                    &current_states.get_frame_index_and_state().get_frame_index(),
                );

                let duration = future_frame_time.duration_since(&last_result_and_time.time_value);

                let partial_duration = now.duration_since(&last_result_and_time.time_value);

                let weight = partial_duration.as_secs_f64() / duration.as_secs_f64();

                if weight < 0.0 {
                    warn!(
                        "Negetive smoothing weight {:?}\nLast Result: {:?}\nFuture State: {:?}",
                        weight, last_result_and_time.time_value, future_frame_time
                    );
                }

                if cfg!(feature = "no-smoothing") {
                    T::to_smooth_result(current_states.get_frame_index_and_state().get_state())
                } else {
                    let arg = SmoothArguments {
                        last_result: &last_result_and_time.smoothed_state,
                        future_state: current_states.get_frame_index_and_state().get_state(),
                        weight,
                    };

                    T::smooth(&arg)
                }
            }
            None => T::to_smooth_result(current_states.get_frame_index_and_state().get_state()),
        };

        let result_and_time = SmoothedStateAndTimeValue {
            smoothed_state: result,
            time_value: now,
        };

        self.last_result = Some(result_and_time);

        let last_result = &self.last_result.as_ref().unwrap();

        let smoothed_state = SmoothedState {
            start_time: current_states.get_start_time(),
            initial_information: current_states.get_initial_information(),
            current_time: &last_result.time_value,
            current_state: &last_result.smoothed_state,
        };

        return Ok(smoothed_state);
    }
}
