use log::{warn, info, trace};
use crate::interface::{State, Input, InputEvent};
use crate::gamemanager::stepmessage::StepMessage;
use crate::threading::{Consumer, Sender, Receiver, channel};
use crate::gametime::{TimeMessage, TimeValue};

pub struct RenderReceiver<StateType, InputType>
    where StateType: State,
          InputType: Input {

    receiver: Receiver<Data<StateType, InputType>>,
    data: Data<StateType, InputType>
}

pub struct Data<StateType, InputType>
    where StateType: State,
          InputType: Input {

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

    pub fn get_step_message(&mut self) -> Option<&StepMessage<StateType, InputType>> {

        self.receiver.try_iter(&mut self.data);

        let now = TimeValue::now();
        //info!("Now: {:?}", now);

        if self.data.step_queue.is_empty() {
            info!("Data is empty");
            return None;

        } else if self.data.latest_time_message.is_some() {

            let latest_time_message = self.data.latest_time_message.as_ref().unwrap();
            let step = latest_time_message.get_step_from_actual_time(now).floor() as usize;

            loop {
                if self.data.step_queue.len() == 1 ||
                    self.data.step_queue[self.data.step_queue.len() - 1].get_step_index() == step {
                    let this_step = self.data.step_queue[self.data.step_queue.len() - 1].get_step_index();

                    if (step as i64 - this_step as i64).abs() > 3 {
                        trace!("Needed step: {:?}, Gotten step: {:?}", step, this_step);
                    }

                    let step = &self.data.step_queue[self.data.step_queue.len() - 1];

                    return Some(step);
                } else {
                    self.data.step_queue.pop();
                }
            }
        } else {
            return None;
        }
    }

}

impl<StateType, InputType> Consumer<StepMessage<StateType, InputType>> for Sender<Data<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    fn accept(&self, step_message: StepMessage<StateType, InputType>) {

        //info!("StepMessage: {:?}", step_message.get_step_index());
        self.send(|data|{
            //info!("StepMessage: {:?}", step_message.get_step_index());
            //insert in reverse sorted order
            match data.step_queue.binary_search_by(|elem| { step_message.cmp(elem) }) {
                Ok(pos) => data.step_queue[pos] = step_message,
                Err(pos) => data.step_queue.insert(pos, step_message)
            }
        }).unwrap();
    }

}

impl<StateType, InputType> Consumer<TimeMessage> for Sender<Data<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |data|{
            data.latest_time_message = Some(time_message);
        }).unwrap();
    }
}