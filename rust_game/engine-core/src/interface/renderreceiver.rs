use std::sync::mpsc::TryRecvError;

use crate::gamemanager::StepMessage;
use crate::gametime::TimeMessage;
use crate::interface::{
    Factory,
    Game,
    GameFactoryTrait,
    GameTrait,
    InitialInformation,
    InterpolationArg,
    InterpolationResult,
};
use commons::factory::FactoryTrait;
use commons::threading::channel::{
    Receiver,
    ReceiverTrait,
    Sender,
};
use commons::time::TimeDuration;
use log::{
    info,
    warn,
};

pub enum RenderReceiverMessage<Game: GameTrait> {
    InitialInformation(InitialInformation<Game>),
    StepMessage(StepMessage<Game>),
    TimeMessage(TimeMessage),
    StopThread,
}

//TODO: make the difference between the render receiver and the Data more clear
pub struct RenderReceiver<GameFactory: GameFactoryTrait> {
    factory: Factory<GameFactory>,
    receiver: Receiver<RenderReceiverMessage<Game<GameFactory>>>,
    data: Data<GameFactory>,
}

struct Data<GameFactory: GameFactoryTrait> {
    factory: Factory<GameFactory>,
    //TODO: use vec deque so that this is more efficient
    step_queue: Vec<StepMessage<Game<GameFactory>>>,
    latest_time_message: Option<TimeMessage>,
    initial_information: Option<InitialInformation<Game<GameFactory>>>,
}

impl<GameFactory: GameFactoryTrait> RenderReceiver<GameFactory> {
    pub fn new(
        factory: Factory<GameFactory>,
    ) -> (Sender<RenderReceiverMessage<Game<GameFactory>>>, Self) {
        let (sender, receiver) = factory.new_channel();

        let data = Data::<GameFactory> {
            factory: factory.clone(),
            step_queue: Vec::new(),
            latest_time_message: None,
            initial_information: None,
        };

        let render_receiver = Self {
            factory,
            receiver,
            data,
        };

        return (sender, render_receiver);
    }

    //TODO: remove timeduration
    //TODO: notify the caller if the channel is disconnected
    pub fn get_step_message(
        self: &mut Self,
    ) -> Option<(TimeDuration, InterpolationResult<GameFactory>)> {
        loop {
            match self.receiver.try_recv() {
                Ok(RenderReceiverMessage::InitialInformation(initial_information)) => {
                    self.data.on_initial_information(initial_information)
                }

                Ok(RenderReceiverMessage::StepMessage(step_message)) => {
                    self.data.on_step_message(step_message)
                }

                Ok(RenderReceiverMessage::TimeMessage(time_message)) => {
                    self.data.on_time_message(time_message)
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

        if self.data.initial_information.is_none() {
            return None;
        } else if self.data.step_queue.is_empty() {
            return None;
        } else if self.data.latest_time_message.is_some() {
            let now = self.factory.get_time_source().now();
            let latest_time_message = self.data.latest_time_message.as_ref().unwrap();
            let mut duration_since_start = latest_time_message.get_duration_since_start(now);
            //used to be floor
            let now_as_fractional_step_index = latest_time_message.get_step_from_actual_time(now);
            let desired_first_step_index = now_as_fractional_step_index.floor() as usize;

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

            if (desired_first_step_index as i64 - first_step.get_step_index() as i64).abs() > 1 {
                warn!(
                    "Needed step: {:?}, Gotten step: {:?}",
                    desired_first_step_index,
                    first_step.get_step_index()
                );
            }

            let mut weight = if second_step.get_step_index() == first_step.get_step_index() {
                1 as f64
            } else {
                (now_as_fractional_step_index - first_step.get_step_index() as f64)
                    / ((second_step.get_step_index() - first_step.get_step_index()) as f64)
            };

            let interpolate = true;
            if !interpolate {
                weight = 1 as f64;
                duration_since_start = (latest_time_message
                    .get_step_duration()
                    .mul_f64(second_step.get_step_index() as f64))
                .clone();
            }

            let arg = InterpolationArg::new(weight, duration_since_start);
            let interpolation_result = Game::<GameFactory>::interpolate(
                self.data.initial_information.as_ref().unwrap(),
                first_step.get_state(),
                second_step.get_state(),
                &arg,
            );

            return Some((duration_since_start, interpolation_result));
        } else {
            return None;
        }
    }

    pub fn get_initial_information(&self) -> &Option<InitialInformation<Game<GameFactory>>> {
        return &self.data.initial_information;
    }
}

impl<GameFactory: GameFactoryTrait> Data<GameFactory> {
    fn drop_steps_before(&mut self, drop_before: usize) {
        while self.step_queue.len() > 2
            && self.step_queue[self.step_queue.len() - 1].get_step_index() < drop_before
        {
            let _dropped = self.step_queue.pop().unwrap();
            //info!("Dropped step: {:?}", dropped.get_step_index());
        }
    }

    fn on_initial_information(
        &mut self,
        initial_information: InitialInformation<Game<GameFactory>>,
    ) {
        self.initial_information = Some(initial_information);
    }

    fn on_step_message(&mut self, step_message: StepMessage<Game<GameFactory>>) {
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

        if let Some(time_message) = self.latest_time_message {
            let now = self.factory.get_time_source().now();

            //TODO: put this in a method
            let latest_step = time_message.get_step_from_actual_time(now).floor() as usize;

            self.drop_steps_before(latest_step);
        }
    }

    fn on_time_message(&mut self, time_message: TimeMessage) {
        //TODO: put this in a method
        let now = self.factory.get_time_source().now();
        let latest_step = time_message.get_step_from_actual_time(now).floor() as usize;

        self.latest_time_message = Some(time_message);

        self.drop_steps_before(latest_step);
    }
}
