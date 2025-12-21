use commons::time::TimeValue;
use log::warn;

use crate::{InitialInformation, StateReceiver, StateReceiverError, game_time::StartTime, smooth::{ExtrapolationArguments, Smooth}};

pub struct SmoothReceiver<T: Smooth> {
    state_receiver: StateReceiver<T::Game>,
    last_result: Option<SmoothResultAndTimeValue<T>>
}

struct SmoothResultAndTimeValue<T: Smooth> {
    result: T::SmoothResult,
    time_value: TimeValue,
}

pub struct SmoothedState<'a, T: Smooth> {
    pub start_time: &'a StartTime,
    pub initial_information: &'a InitialInformation<T::Game>,
    pub current_time: &'a TimeValue,
    pub current_state: &'a T::SmoothResult
}

impl<T: Smooth> SmoothReceiver<T> {

    pub fn new(state_receiver: StateReceiver<T::Game>) -> Self {
        Self {
            state_receiver,
            last_result: None,
        }
    }

    pub fn get_smoothed_state(&mut self) -> Result<SmoothedState<T>, StateReceiverError> {

        let current_states = self.state_receiver.get_step_message()?;

        let now = current_states.time_source.now();

        let result = match &self.last_result {
            Some(last_result_and_time) => {

                let future_frame_time = current_states.start_time.get_frame_time_of_occurence(
                    current_states.initial_information.get_server_config().get_frame_duration(),
                    &current_states.next_frame.get_frame_index()
                );

                let duration = future_frame_time.duration_since(&last_result_and_time.time_value);
                
                let partial_duration = now.duration_since(&last_result_and_time.time_value);

                let weight = partial_duration.as_secs_f64() / duration.as_secs_f64();

                if weight < 0.0 {
                    warn!("Negetive smoothing weight {:?}\nLast Result: {:?}\nFuture State: {:?}", weight, last_result_and_time.time_value, future_frame_time);
                }
                
                /*

                let arg = ExtrapolationArguments {
                    last_result: &last_result_and_time.result,
                    future_state: current_states.next_frame.get_state(),
                    weight,
                };

                T::smooth(&arg)
                */
                T::to_smooth_result(current_states.next_frame.get_state())
            },
            None => T::to_smooth_result(current_states.next_frame.get_state()),
        };

        let result_and_time = SmoothResultAndTimeValue {
            result,
            time_value: now,
        };

        self.last_result = Some(result_and_time);

        let last_result = &self.last_result.as_ref().unwrap();

        let smoothed_state = SmoothedState {
            start_time: current_states.start_time,
            initial_information: current_states.initial_information,
            current_time: &last_result.time_value,
            current_state: &last_result.result,
        };
        
        return Ok(smoothed_state);

    }

}