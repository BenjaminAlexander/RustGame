use std::sync::mpsc::TryRecvError;

use crate::game_time::{
    FrameIndex,
    StartTime,
};
use crate::gamemanager::StepMessage;
use crate::interface::{
    GameTrait,
    InitialInformation,
    InterpolationArg,
};
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

pub enum RenderReceiverMessage<Game: GameTrait> {
    InitialInformation(InitialInformation<Game>),
    StepMessage(StepMessage<Game>),
    //TODO: rename and document these
    StartTime(StartTime),
    FrameIndex(FrameIndex),
    StopThread,
}

//TODO: make the difference between the render receiver and the Data more clear
pub struct RenderReceiver<Game: GameTrait> {
    time_source: TimeSource,
    receiver: Receiver<RenderReceiverMessage<Game>>,
    data: Data<Game>,
}

struct Data<Game: GameTrait> {
    time_source: TimeSource,
    start_time: Option<StartTime>,
    //TODO: use vec deque so that this is more efficient
    step_queue: Vec<StepMessage<Game>>,
    latest_frame_index: Option<FrameIndex>,
    initial_information: Option<InitialInformation<Game>>,
}

impl<Game: GameTrait> RenderReceiver<Game> {
    pub fn new(factory: &Factory) -> (Sender<RenderReceiverMessage<Game>>, Self) {
        let (sender, receiver) = factory.new_channel();

        let data = Data::<Game> {
            time_source: factory.get_time_source().clone(),
            start_time: None,
            step_queue: Vec::new(),
            latest_frame_index: None,
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
    pub fn get_step_message(self: &mut Self) -> Option<(TimeDuration, Game::InterpolationResult)> {
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

                Ok(RenderReceiverMessage::FrameIndex(frame_index)) => {
                    self.data.on_frame_index(frame_index)
                }

                Ok(RenderReceiverMessage::StopThread) => {
                    info!("Thread commanded to stop, but not stopping...");
                    //TODO: notify the caller if the channel is disconnected
                    break;
                }

                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    info!("Channel disconnected.");
                    //TODO: notify the caller if the channel is disconnected
                    break;
                }
            }
        }

        if self.data.step_queue.is_empty() {
            return None;
        }

        let start_time = match &self.data.start_time {
            Some(start_time) => start_time,
            None => return None,
        };

        let initial_information = match &self.data.initial_information {
            Some(initial_information) => initial_information,
            None => return None,
        };

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

        let first_step = &self.data.step_queue[self.data.step_queue.len() - 1];
        let second_step = if self.data.step_queue.len() >= 2 {
            &self.data.step_queue[self.data.step_queue.len() - 2]
        } else {
            first_step
        };

        if first_step.get_step_index() + 1 != second_step.get_step_index() {
            warn!(
                "Interpolating from non-sequential steps: {:?}, {:?}",
                first_step.get_step_index(),
                second_step.get_step_index()
            );
        }

        if (desired_first_step_index.usize() as i64 - first_step.get_step_index().usize() as i64)
            .abs()
            > 1
        {
            warn!(
                "Needed step: {:?}, Gotten step: {:?}",
                desired_first_step_index,
                first_step.get_step_index()
            );
        }

        let mut weight = if second_step.get_step_index() == first_step.get_step_index() {
            1 as f64
        } else {
            let fractional_frame_index = start_time.get_fractional_frame_index(
                initial_information.get_server_config().get_frame_duration(),
                &now,
            );
            (fractional_frame_index - first_step.get_step_index().usize() as f64)
                / ((second_step.get_step_index().usize() - first_step.get_step_index().usize())
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
    }

    pub fn get_initial_information(&self) -> &Option<InitialInformation<Game>> {
        return &self.data.initial_information;
    }
}

impl<Game: GameTrait> Data<Game> {
    fn drop_steps_before(&mut self, drop_before: FrameIndex) {
        while self.step_queue.len() > 2
            && self.step_queue[self.step_queue.len() - 1].get_step_index() < drop_before
        {
            let _dropped = self.step_queue.pop().unwrap();
            //info!("Dropped step: {:?}", dropped.get_step_index());
        }
    }

    fn on_initial_information(&mut self, initial_information: InitialInformation<Game>) {
        self.initial_information = Some(initial_information);
    }

    fn on_step_message(&mut self, step_message: StepMessage<Game>) {
        if !self.step_queue.is_empty()
            && self.step_queue[0].get_step_index() + 1 < step_message.get_step_index()
        {
            warn!(
                "Received steps out of order.  Waiting for {:?} but got {:?}.",
                self.step_queue[0].get_step_index() + 1,
                step_message.get_step_index()
            );
        }

        //info!("StepMessage: {:?}", step_message.get_step_index());
        //insert in reverse sorted order
        match self
            .step_queue
            .binary_search_by(|elem| step_message.cmp(elem))
        {
            Ok(pos) => self.step_queue[pos] = step_message,
            Err(pos) => self.step_queue.insert(pos, step_message),
        }

        //TODO: this while block seems strange
        if let Some(latest_frame_index) = self.latest_frame_index {
            self.drop_steps_before(latest_frame_index);
        }
    }

    fn on_start_time(&mut self, start_time: StartTime) {
        self.start_time = Some(start_time);
    }

    fn on_frame_index(&mut self, frame_index: FrameIndex) {
        self.latest_frame_index = Some(frame_index);
        self.drop_steps_before(frame_index);
    }
}
