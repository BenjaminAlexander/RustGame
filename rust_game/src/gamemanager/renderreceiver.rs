use log::{warn, info, trace};
use crate::interface::{State, Input, InputEvent};
use crate::gamemanager::stepmessage::StepMessage;
use crate::threading::{Consumer, Sender, Receiver, channel};
use crate::gametime::{TimeMessage, TimeValue, TimeDuration};

pub struct RenderReceiver<StateType, InputType>
    where StateType: State,
          InputType: Input {

    receiver: Receiver<Data<StateType, InputType>>,
    data: Data<StateType, InputType>
}

pub struct Data<StateType, InputType>
    where StateType: State,
          InputType: Input {

    //TODO: use vec deque so that this is more efficient
    step_queue: Vec<StepMessage<StateType, InputType>>,
    latest_time_message: Option<TimeMessage>,

    //metrics
    next_expected_step_index: usize
}

impl<StateType, InputType> Data<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn drop_steps_before(&mut self, drop_before: usize) {
        while self.step_queue.len() > 2 &&
            self.step_queue[self.step_queue.len() - 1].get_step_index() < drop_before {

            let dropped = self.step_queue.pop().unwrap();
            //info!("Dropped step: {:?}", dropped.get_step_index());
        }
    }
}

impl<StateType, InputType> RenderReceiver<StateType, InputType>
    where StateType: State,
          InputType: Input {

    pub fn new() -> (Sender<Data<StateType, InputType>>, Self) {
        let (sender, receiver) = channel::<Data<StateType, InputType>>();

        let render_receiver = Self{
            receiver,
            data: Data {
                step_queue: Vec::new(),
                latest_time_message: None,
                next_expected_step_index: 0
            }
        };

        return (sender, render_receiver);
    }

    pub fn get_step_message(&mut self) -> Option<(TimeDuration, &StepMessage<StateType, InputType>)> {

        self.receiver.try_iter(&mut self.data);

        if self.data.step_queue.is_empty() {
            return None;

        } else if self.data.latest_time_message.is_some() {

            let now = TimeValue::now();
            let latest_time_message = self.data.latest_time_message.as_ref().unwrap();
            let duration_since_start = latest_time_message.get_duration_since_start(now);
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

            return Some((duration_since_start, second_step));

        } else {
            return None;
        }
    }

}

//TODO: dropping steps in a strange way

impl<StateType, InputType> Consumer<StepMessage<StateType, InputType>> for Sender<Data<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    fn accept(&self, step_message: StepMessage<StateType, InputType>) {

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

impl<StateType, InputType> Consumer<TimeMessage> for Sender<Data<StateType, InputType>>
    where StateType: State,
          InputType: Input {

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