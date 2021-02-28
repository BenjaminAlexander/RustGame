use log::{warn, info, trace};
use crate::interface::{State, Input, InputEvent, InterpolationArg, Interpolate, InterpolationResult};
use crate::gamemanager::stepmessage::StepMessage;
use crate::threading::{Consumer, Sender, Receiver, channel};
use crate::gametime::{TimeMessage, TimeValue, TimeDuration};
use std::marker::PhantomData;
use crate::messaging::InitialInformation;

pub struct RenderReceiver<StateType, InterpolateType, InterpolatedType>
    where StateType: State,
          InterpolateType: Interpolate<StateType, InterpolatedType>,
          InterpolatedType: InterpolationResult {

    receiver: Receiver<Data<StateType>>,
    data: Data<StateType>,
    phantom: PhantomData<(InterpolateType, InterpolatedType)>
}

pub struct Data<StateType>
    where StateType: State {

    //TODO: use vec deque so that this is more efficient
    step_queue: Vec<StepMessage<StateType>>,
    latest_time_message: Option<TimeMessage>,
    initial_information: Option<InitialInformation<StateType>>,

    //metrics
    next_expected_step_index: usize
}

impl<StateType> Data<StateType>
    where StateType: State {

    fn drop_steps_before(&mut self, drop_before: usize) {
        while self.step_queue.len() > 2 &&
            self.step_queue[self.step_queue.len() - 1].get_step_index() < drop_before {

            let dropped = self.step_queue.pop().unwrap();
            //info!("Dropped step: {:?}", dropped.get_step_index());
        }
    }
}

impl<StateType, InterpolateType, InterpolatedType> RenderReceiver<StateType, InterpolateType, InterpolatedType>
    where StateType: State,
          InterpolateType: Interpolate<StateType, InterpolatedType>,
          InterpolatedType: InterpolationResult {

    pub fn new() -> (Sender<Data<StateType>>, Self) {
        let (sender, receiver) = channel::<Data<StateType>>();

        let render_receiver = Self{
            receiver,
            phantom: PhantomData,
            data: Data {
                step_queue: Vec::new(),
                latest_time_message: None,
                initial_information: None,
                next_expected_step_index: 0
            }
        };

        return (sender, render_receiver);
    }

    //TODO: remove timeduration
    pub fn get_step_message(&mut self) -> Option<(TimeDuration, InterpolatedType)> {

        self.receiver.try_iter(&mut self.data);

        if self.data.initial_information.is_none() {
            return None;

        } else if self.data.step_queue.is_empty() {
            return None;

        } else if self.data.latest_time_message.is_some() {

            let now = TimeValue::now();
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
                warn!("Interpolating from non-sequential steps: {:?}, {:?}",
                      first_step.get_step_index(), second_step.get_step_index());
            }

            if (desired_first_step_index as i64 - first_step.get_step_index() as i64).abs() > 1 {
                warn!("Needed step: {:?}, Gotten step: {:?}", desired_first_step_index, first_step.get_step_index());
            }

            let mut weight = if second_step.get_step_index() == first_step.get_step_index() {
                1 as f64
            } else {
                (now_as_fractional_step_index - first_step.get_step_index() as f64) /
                    ((second_step.get_step_index() - first_step.get_step_index()) as f64)
            };

            let interpolate = true;
            if !interpolate {
                weight = 1 as f64;
                duration_since_start = (latest_time_message.get_step_duration() * second_step.get_step_index() as i64).clone();
            }

            let arg = InterpolationArg::new(weight, duration_since_start);
            let interpolation_result = InterpolateType::interpolate(
                self.data.initial_information.as_ref().unwrap(),
                first_step.get_state(),
                second_step.get_state(),
                &arg);

            return Some((duration_since_start, interpolation_result));

        } else {
            return None;
        }
    }

}

impl<StateType> Consumer<StepMessage<StateType>> for Sender<Data<StateType>>
    where StateType: State {

    fn accept(&self, step_message: StepMessage<StateType>) {

        //info!("StepMessage: {:?}", step_message.get_step_index());
        self.send(|data|{

            if data.next_expected_step_index != step_message.get_step_index() {
                warn!("Received steps out of order.  Waiting for {:?} but got {:?}.",
                      data.next_expected_step_index, step_message.get_step_index());
            }
            data.next_expected_step_index = step_message.get_step_index() + 1;

            //info!("StepMessage: {:?}", step_message.get_step_index());
            //insert in reverse sorted order
            match data.step_queue.binary_search_by(|elem| { step_message.cmp(elem) }) {
                Ok(pos) => data.step_queue[pos] = step_message,
                Err(pos) => data.step_queue.insert(pos, step_message)
            }

            if let Some(time_message) = data.latest_time_message {
                let now = TimeValue::now();

                //TODO: put this in a method
                let latest_step = time_message.get_step_from_actual_time(now).floor() as usize;

                data.drop_steps_before(latest_step);
            }

        }).unwrap();
    }

}

impl<StateType> Consumer<TimeMessage> for Sender<Data<StateType>>
    where StateType: State {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |data|{

            //TODO: put this in a method
            let now = TimeValue::now();
            let latest_step = time_message.get_step_from_actual_time(now).floor() as usize;

            data.latest_time_message = Some(time_message);

            data.drop_steps_before(latest_step);

        }).unwrap();
    }
}

impl<StateType> Consumer<InitialInformation<StateType>> for Sender<Data<StateType>>
    where StateType: State {

    fn accept(&self, initial_information: InitialInformation<StateType>) {
        self.send(|data|{
            data.initial_information = Some(initial_information);
        }).unwrap();
    }
}