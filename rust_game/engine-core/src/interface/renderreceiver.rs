use std::sync::mpsc::TryRecvError;

use crate::game_time::{
    StartTime,
};
use crate::interface::{
    GameTrait,
    InitialInformation,
    InterpolationArg,
};
use crate::messaging::FrameIndexAndState;
use commons::real_time::{
    Factory,
    Receiver,
    Sender,
    TimeSource,
};
use commons::time::TimeDuration;
use log::{
    info,
    warn,
};

pub struct CurrentStates<'a, Game: GameTrait> {
    pub time_source: &'a TimeSource,
    pub start_time: &'a StartTime,
    pub initial_information: &'a InitialInformation<Game>,
    pub current_frame: &'a FrameIndexAndState<Game>,
    pub next_frame: &'a FrameIndexAndState<Game>,
}

pub enum StateReceiverError {
    Disconnected,
    StateNoYetAvailable
}

//TODO: make the difference between the render receiver and the Data more clear
pub struct StateReceiver<Game: GameTrait> {
    time_source: TimeSource,
    receiver: Receiver<RenderReceiverMessage<Game>>,
    data: Data<Game>,
}

struct Data<Game: GameTrait> {
    start_time: Option<StartTime>,
    current_frame: Option<FrameIndexAndState<Game>>,
    next_frame: Option<FrameIndexAndState<Game>>,
    initial_information: Option<InitialInformation<Game>>,
}

impl<Game: GameTrait> StateReceiver<Game> {
    pub fn new(factory: &Factory) -> (Sender<RenderReceiverMessage<Game>>, Self) {
        let (sender, receiver) = factory.new_channel();

        let data = Data::<Game> {
            start_time: None,
            current_frame: None,
            next_frame: None,
            initial_information: None,
        };

        let render_receiver = Self {
            time_source: factory.get_time_source().clone(),
            receiver,
            data,
        };

        return (sender, render_receiver);
    }

    //TODO: remove timeduration
    //TODO: notify the caller if the channel is disconnected
    pub fn get_step_message(&mut self) -> Result<CurrentStates<Game>, StateReceiverError> {
        loop {
            match self.receiver.try_recv() {
                Ok(RenderReceiverMessage::InitialInformation(initial_information)) => {
                    self.data.on_initial_information(initial_information)
                }

                Ok(RenderReceiverMessage::StepMessage(step_message)) => {
                    self.data.on_step_message(step_message)
                }

                Ok(RenderReceiverMessage::StartTime(start_time)) => {
                    self.data.on_start_time(start_time)
                }

                Ok(RenderReceiverMessage::StopThread) => {
                    info!("Thread commanded to stop, but not stopping...");
                    //TODO: notify the caller if the channel is disconnected
                    break;
                }

                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    info!("Channel disconnected.");
                    return Err(StateReceiverError::Disconnected);
                }
            }
        }

        if self.data.next_frame.is_none() {
            return Err(StateReceiverError::StateNoYetAvailable);
        }

        let start_time = match &self.data.start_time {
            Some(start_time) => start_time,
            None => return Err(StateReceiverError::StateNoYetAvailable),
        };

        let initial_information = match &self.data.initial_information {
            Some(initial_information) => initial_information,
            None => return Err(StateReceiverError::StateNoYetAvailable),
        };

        let next_frame = match &self.data.next_frame {
            Some(second_step) => second_step,
            None => return Err(StateReceiverError::StateNoYetAvailable),
        };

        let current_frame = match &self.data.current_frame {
            Some(first_step) => first_step,
            None => next_frame,
        };

        return Ok(CurrentStates{
            time_source: &self.time_source,
            start_time,
            initial_information,
            current_frame,
            next_frame,
        });
/*

        let now = self.time_source.now();
        //let latest_time_message = self.data.latest_time_message.as_ref().unwrap();
        //let mut duration_since_start = latest_time_message.get_duration_since_start(now);
        //used to be floor
        //let now_as_fractional_step_index = latest_time_message.get_step_from_actual_time(now);

        //TODO: should probably be interpolating between the current frame (nearest past), to next (nearest future)
        let desired_first_step_index = start_time.get_frame_index(
            initial_information.get_server_config().get_frame_duration(),
            &now,
        );

        if first_step.get_frame_index() + 1 != second_step.get_frame_index() {
            warn!(
                "Interpolating from non-sequential steps: {:?}, {:?}",
                first_step.get_frame_index(),
                second_step.get_frame_index()
            );
        }

        if (desired_first_step_index.usize() as i64 - first_step.get_frame_index().usize() as i64)
            .abs()
            > 1
        {
            warn!(
                "Needed step: {:?}, Gotten step: {:?}",
                desired_first_step_index,
                first_step.get_frame_index()
            );
        }

        let mut weight = if second_step.get_frame_index() == first_step.get_frame_index() {
            1 as f64
        } else {
            let fractional_frame_index = start_time.get_fractional_frame_index(
                initial_information.get_server_config().get_frame_duration(),
                &now,
            );
            (fractional_frame_index - first_step.get_frame_index().usize() as f64)
                / ((second_step.get_frame_index().usize() - first_step.get_frame_index().usize())
                    as f64)
        };

        let interpolate = true;
        if !interpolate {
            weight = 1 as f64;
        }

        //TODO: this duration since start thing seems strange
        // The render receiver should probably just expose the start time
        let duration_since_start = now.duration_since(start_time.get_time_value());

        let arg = InterpolationArg::new(weight, duration_since_start);
        let interpolation_result = Game::interpolate(
            self.data.initial_information.as_ref().unwrap(),
            first_step.get_state(),
            second_step.get_state(),
            &arg,
        );

        return Some((duration_since_start, interpolation_result));
        */
    }

    pub fn get_initial_information(&self) -> &Option<InitialInformation<Game>> {
        return &self.data.initial_information;
    }
}

impl<Game: GameTrait> Data<Game> {
    fn on_initial_information(&mut self, initial_information: InitialInformation<Game>) {
        self.initial_information = Some(initial_information);
    }

    fn on_step_message(&mut self, frame_index_and_state: FrameIndexAndState<Game>) {

        let next_frame = match &mut self.next_frame {
            Some(next_frame) => next_frame,
            None => {
                self.next_frame = Some(frame_index_and_state);
                return;
            },
        };

        if next_frame.get_frame_index() == frame_index_and_state.get_frame_index() {
            *next_frame = frame_index_and_state;
            return;
        }

        if next_frame.get_frame_index() < frame_index_and_state.get_frame_index() {
            self.current_frame = self.next_frame.take();
            self.next_frame = Some(frame_index_and_state);
            return;
        }

        let current_frame = match &mut self.current_frame {
            Some(current_frame) => current_frame,
            None => {
                self.current_frame = Some(frame_index_and_state);
                return;
            },
        };

        if current_frame.get_frame_index() <= frame_index_and_state.get_frame_index() {
            *current_frame = frame_index_and_state;
        }
    }

    fn on_start_time(&mut self, start_time: StartTime) {
        self.start_time = Some(start_time);
    }
}

pub enum RenderReceiverMessage<Game: GameTrait> {
    InitialInformation(InitialInformation<Game>),
    //TODO: rename
    StepMessage(FrameIndexAndState<Game>),
    //TODO: rename and document these
    StartTime(StartTime),
    StopThread,
}
