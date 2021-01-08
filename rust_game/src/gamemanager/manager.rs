use log::{warn};
use crate::messaging::{StateMessage, InputMessage};
use std::collections::VecDeque;
use crate::interface::{Input, State};
use crate::threading::{ChannelDrivenThread, Sender, Consumer};
use crate::gamemanager::step::Step;

pub struct Manager<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    sequence_of_queue_index_0: usize,

    //TODO: send requested state emediately if available
    sequence_to_update_to: usize,
    player_count: usize,
    //New states at the back, old at the front (index 0)
    states: VecDeque<Step<StateType, InputType>>
}

impl<StateType, InputType> Manager<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn new(player_count: usize, initialState: StateMessage<StateType>) -> Self {
        let mut new_manager: Self = Self{
            player_count,
            states: VecDeque::new(),
            sequence_of_queue_index_0: initialState.get_sequence(),
            sequence_to_update_to: 0,
        };

        let mut first_state = Step::blank(initialState.get_sequence(), player_count);
        first_state.set_final_state(initialState);
        new_manager.states.push_back(first_state);

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
        while self.states.len() <= index {
            let blank = Step::blank(self.states.len() + self.sequence_of_queue_index_0, self.player_count);
            self.states.push_back(blank);
        }
    }

    fn get_state(&mut self, sequence: usize) -> Option<&mut Step<StateType, InputType>> {
        match self.get_index_of_sequence(sequence) {
            None => {
                warn!("Cannot get a state from the past.");
                None
            } Some(index) => {
                self.pad_to_index(index);
                Some(&mut self.states[index])
            }
        }
    }
}

impl<StateType, InputType> ChannelDrivenThread<()> for Manager<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn on_none_pending(&mut self) -> Option<()> {

        let index_to_update_to = self.get_index_of_sequence(self.sequence_to_update_to).unwrap();

        let mut requested_state_changed = false;
        let mut send_completed_index = false;
        let mut newest_completed_index = 0;
        let mut i = 0;

        while (i == newest_completed_index && self.states[i].has_all_inputs()) ||
                (i < index_to_update_to) {

            //Make sure the next state exists
            if  self.states.len() <= i + 1 {
                self.states.push_back(Step::blank(i + 1 + self.sequence_of_queue_index_0, self.player_count))
            }

            let mut was_updated = false;
            if !self.states[i + 1].is_state_final() {
                let next_step = self.states[i].calculate_next_state();
                if next_step.is_some() {
                    self.states[i + 1].set_calculated_state(next_step.unwrap());
                    was_updated = true;
                }
            }

            if i + 1 == index_to_update_to && was_updated {
                requested_state_changed = true;
                //TODO: send requested state
            }

            if newest_completed_index == i && self.states[i].has_all_inputs() {
                newest_completed_index = i + 1;

                if was_updated {
                    send_completed_index = true;
                    //TODO: Send completed state
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

}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |manager|{
            match manager.get_state(input_message.get_sequence()) {
                None => warn!("A input from past was received.  The buffer can only extend into the future.  This input will be dropped."),
                Some(sequence) => sequence.set_input(input_message)
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