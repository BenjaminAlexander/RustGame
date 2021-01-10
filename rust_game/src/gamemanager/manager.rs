use log::{warn};
use crate::messaging::{StateMessage, InputMessage};
use std::collections::VecDeque;
use crate::interface::{Input, State};
use crate::threading::{ConsumerList, ChannelDrivenThread, Sender, Consumer};
use crate::gamemanager::step::Step;
use crate::gamemanager::stepmessage::StepMessage;

pub struct Manager<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    sequence_of_queue_index_0: usize,

    //TODO: send requested state immediately if available
    requested_sequence: usize,
    player_count: usize,
    //New states at the back, old at the front (index 0)
    steps: VecDeque<Step<StateType, InputType>>,
    requested_step_consumer_list: ConsumerList<StepMessage<StateType, InputType>>,
    completed_step_consumer_list: ConsumerList<StateMessage<StateType>>

}

impl<StateType, InputType> Manager<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn new(player_count: usize, initialState: StateMessage<StateType>) -> Self {
        let mut new_manager: Self = Self{
            player_count,
            steps: VecDeque::new(),
            sequence_of_queue_index_0: initialState.get_sequence(),
            requested_sequence: 0,
            requested_step_consumer_list: ConsumerList::new(),
            completed_step_consumer_list: ConsumerList::new()
        };

        let mut first_state = Step::blank(initialState.get_sequence(), player_count);
        first_state.set_final_state(initialState);
        new_manager.steps.push_back(first_state);

        return new_manager;
    }

    fn get_index_of_sequence(&self, sequence: usize) -> Option<usize> {
        if sequence < self.sequence_of_queue_index_0 {
            None
        } else {
            Some(sequence - self.sequence_of_queue_index_0)
        }
    }

    fn pad_to_index(&mut self, index: usize) {
        while self.steps.len() <= index {
            let blank = Step::blank(self.steps.len() + self.sequence_of_queue_index_0, self.player_count);
            self.steps.push_back(blank);
        }
    }

    fn get_state(&mut self, sequence: usize) -> Option<&mut Step<StateType, InputType>> {
        match self.get_index_of_sequence(sequence) {
            None => {
                warn!("Cannot get a state from the past.");
                None
            } Some(index) => {
                self.pad_to_index(index);
                Some(&mut self.steps[index])
            }
        }
    }
}

impl<StateType, InputType> ChannelDrivenThread<()> for Manager<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn on_none_pending(&mut self) -> Option<()> {

        let index_to_update_to = self.get_index_of_sequence(self.requested_sequence).unwrap();

        let mut newest_completed_index = 0;
        let mut i = 0;

        while (i == newest_completed_index && self.steps[i].has_all_inputs()) ||
                (i < index_to_update_to + 1) {

            //Make sure the next state exists
            if  self.steps.len() <= i + 1 {
                self.steps.push_back(Step::blank(i + 1 + self.sequence_of_queue_index_0, self.player_count))
            }

            let mut was_updated = false;
            if !self.steps[i + 1].is_state_final() {
                let next_step = self.steps[i].calculate_next_state();
                if next_step.is_some() {
                    self.steps[i + 1].set_calculated_state(next_step.unwrap());
                    was_updated = true;
                }
            }

            if i + 1 >= index_to_update_to && was_updated {
                self.requested_step_consumer_list.accept(&self.steps[i + 1].get_message());
            }

            if newest_completed_index == i && self.steps[i].has_all_inputs() {
                newest_completed_index = i + 1;

                if was_updated {
                    let message = StateMessage::new(i + 1, self.steps[i + 1].get_state().unwrap().clone());
                    self.completed_step_consumer_list.accept(&message)
                }
            }

            i = i + 1;
        }
        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn add_requested_step_consumer<T>(&self, consumer: T)
        where T: Consumer<StepMessage<StateType, InputType>> {
        self.send(move |manager|{
            manager.requested_step_consumer_list.add_consumer(consumer);
        }).unwrap();
    }

    pub fn add_completed_step_consumer<T>(&self, consumer: T)
        where T: Consumer<StateMessage<StateType>> {
        self.send(move |manager|{
            manager.completed_step_consumer_list.add_consumer(consumer);
        }).unwrap();
    }

    pub fn drop_step(sequence :usize)
}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |manager|{
            match manager.get_state(input_message.get_sequence()) {
                None => warn!("A input from past was received.  The buffer can only extend into the future.  This input will be dropped."),
                Some(step) => step.set_input(input_message)
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<StateMessage<StateType>> for Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, mut state_message: StateMessage<StateType>) {
        self.send(move |manager|{
            match manager.get_state(state_message.get_sequence()) {
                None => warn!("A state from past was received.  The buffer can only extend into the future.  This state will be dropped."),
                Some(step) => step.set_final_state(state_message),
            }
        }).unwrap();
    }
}