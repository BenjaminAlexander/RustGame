use log::{warn};
use crate::messaging::{StateMessage, InputMessage};
use std::collections::VecDeque;
use crate::interface::{Input, State};
use crate::threading::{ChannelDrivenThread, Sender, Consumer};
use crate::gamemanager::step::Step;

pub struct Manager<StateType, InputType>
    where InputType: Input,
          StateType: State {

    sequence_of_queue_index_0: usize,
    sequence_to_update_to: usize,
    player_count: usize,
    //New states at the back, old at the front (index 0)
    states: VecDeque<Step<StateType, InputType>>
}

impl<StateType, InputType> Manager<StateType, InputType>
    where InputType: Input,
          StateType: State {

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
          StateType: State {

    fn on_none_pending(&mut self) -> Option<()> {
        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State {

}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |manager|{
            match manager.get_state(input_message.get_sequence()) {
                None => warn!("Failed to add state."),
                Some(sequence) => sequence.set_input(input_message)
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<StateMessage<StateType>> for Sender<Manager<StateType, InputType>>
    where InputType: Input,
          StateType: State {

    fn accept(&self, mut state_message: StateMessage<StateType>) {
        self.send(move |manager|{
            match manager.get_state(state_message.get_sequence()) {
                None => warn!("A state from past was received.  The buffer can only extend into the future.  This state will be dropped."),
                Some(step) => step.set_final_state(state_message),
            }
        }).unwrap();
    }
}