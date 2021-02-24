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
    latest_time_message: Option<TimeMessage>
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
                latest_time_message: None
            }
        };

        return (sender, render_receiver);
    }

    pub fn get_step_message(&mut self) -> Option<(TimeDuration, &StepMessage<StateType, InputType>)> {

        self.receiver.try_iter(&mut self.data);

        let now = TimeValue::now();
        //info!("Now: {:?}", now);

        if self.data.step_queue.is_empty() {
            info!("Data is empty");
            return None;

        } else if self.data.latest_time_message.is_some() {

            let latest_time_message = self.data.latest_time_message.as_ref().unwrap();
            let duration_since_start = latest_time_message.get_duration_since_start(now);
            //used to be floor
            let needed_step = latest_time_message.get_step_from_actual_time(now).ceil() as usize;

            let step = if self.data.step_queue.len() > 2 {
                &self.data.step_queue[self.data.step_queue.len() - 2]
            } else {
                &self.data.step_queue[self.data.step_queue.len() - 1]
            };

            //let this_step = self.data.step_queue[self.data.step_queue.len() - 1].get_step_index();

            info!("Needed step: {:?}, Gotten step: {:?}, len: {:?}", needed_step, step.get_step_index(), self.data.step_queue.len());

            let mut x = "Steps: ".to_owned();
            for sm in &self.data.step_queue {
                x = x + &*sm.get_step_index().to_string() + ", ";
            }

            info!("Available: {:?}", x);

            if (needed_step as i64 - step.get_step_index() as i64).abs() > 3 {
                trace!("Needed step: {:?}, Gotten step: {:?}", needed_step, step.get_step_index());
            }

            //let step = &self.data.step_queue[self.data.step_queue.len() - 1];

            return Some((duration_since_start, step));

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

            let now = TimeValue::now();

            if let Some(time_message) = data.latest_time_message {
                //TODO: put this in a method
                let latest_step = time_message.get_step_from_actual_time(now).floor() as usize;

                if latest_step > step_message.get_step_index() {
                    return;
                }
            }

            //info!("StepMessage: {:?}", step_message.get_step_index());
            //insert in reverse sorted order
            match data.step_queue.binary_search_by(|elem| { step_message.cmp(elem) }) {
                Ok(pos) => data.step_queue[pos] = step_message,
                Err(pos) => data.step_queue.insert(pos, step_message)
            }

            if let Some(time_message) = data.latest_time_message {
                //TODO: put this in a method
                let latest_step = time_message.get_step_from_actual_time(now).floor() as usize;
                //TODO: put this in a method
                while data.step_queue.len() > 2 &&
                    data.step_queue[data.step_queue.len() - 1].get_step_index() < latest_step {
                    data.step_queue.pop();
                }
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

            //TODO: put this in a method
            while data.step_queue.len() > 2 &&
                data.step_queue[data.step_queue.len() - 1].get_step_index() < latest_step {

                data.step_queue.pop();
            }

        }).unwrap();
    }
}