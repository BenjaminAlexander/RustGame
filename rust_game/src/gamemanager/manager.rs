use log::{warn, trace, info};
use crate::messaging::{StateMessage, InputMessage, InitialInformation};
use std::collections::VecDeque;
use crate::interface::{Input, State, InputEvent};
use crate::threading::{ConsumerList, ChannelDrivenThread, Sender, Consumer};
use crate::gamemanager::step::Step;
use crate::gamemanager::stepmessage::StepMessage;
use crate::gametime::{TimeMessage, TimeReceived, TimeDuration};

pub struct Manager<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    drop_steps_before: usize,
    //TODO: send requested state immediately if available
    requested_step: usize,
    player_count: Option<usize>,
    //New states at the back, old at the front (index 0)
    steps: VecDeque<Step<StateType, InputType>>,
    requested_step_consumer_list: ConsumerList<StepMessage<StateType, InputType>>,
    completed_step_consumer_list: ConsumerList<StateMessage<StateType>>,
    grace_period: TimeDuration
}

impl<StateType, InputType> Manager<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn new(grace_period: TimeDuration) -> Self {
        Self{
            player_count: None,
            steps: VecDeque::new(),
            requested_step: 0,
            drop_steps_before: 0,
            requested_step_consumer_list: ConsumerList::new(),
            completed_step_consumer_list: ConsumerList::new(),
            grace_period
        }
    }

    fn get_state(&mut self, step_index: usize) -> &mut Step<StateType, InputType> {

        if self.steps.is_empty() {
            let step = Step::blank(step_index);
            self.steps.push_back(step);
            return &mut self.steps[0];
        } else if step_index <= self.steps[0].get_step_index() {
            loop {
                let zero_index = self.steps[0].get_step_index();
                if zero_index == step_index {
                    return &mut self.steps[0];
                } else {
                    self.steps.push_front(Step::blank(zero_index - 1))
                }
            }
        } else {
            let index_to_get = step_index - self.steps[0].get_step_index();
            while self.steps.len() <= index_to_get {
                self.steps.push_back(Step::blank(self.steps[self.steps.len() - 1].get_step_index() + 1));
            }
            return &mut self.steps[index_to_get];
        }
    }

    fn handle_state_message(&mut self, state_message: StateMessage<StateType>) {
        let step = self.get_state(state_message.get_sequence());
        step.set_final_state(state_message);
    }

    fn drop_steps_before(&mut self, step :usize) {
        trace!("Setting drop_steps_before: {:?}", step);
        self.drop_steps_before = step;
        if self.requested_step < self.drop_steps_before {
            warn!{"Requested step is earlier than drop step: {:?}", self.drop_steps_before};
            self.set_requested_step(step);
        }
    }

    fn set_requested_step(&mut self, step: usize) {
        trace!("Setting requested_step: {:?}", step);
        self.requested_step = step;
    }

    fn send_messages(&mut self, step_index: usize) {
        let changed_message_option = self.steps[step_index].get_changed_message();
        if changed_message_option.is_some() {
            self.requested_step_consumer_list.accept(&changed_message_option.unwrap());
        }

        let complete_message_option = self.steps[step_index].get_complete_message();
        if complete_message_option.is_some() {
            self.completed_step_consumer_list.accept(&complete_message_option.unwrap());
        }
    }
}

impl<StateType, InputType> ChannelDrivenThread<()> for Manager<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    fn on_none_pending(&mut self) -> Option<()> {

        if self.steps.is_empty() {
            trace!("Steps is empty");
            return None;
        }

        let mut current: usize = 0;

        self.send_messages(current);

        let player_count = match self.player_count {
            None => usize::MAX,
            Some(count) => count
        };

        while (self.steps[current].is_complete() && self.steps[current].get_input_count() == player_count) ||
                (self.steps[current].get_step_index() < self.requested_step) {

            let next = current + 1;
            self.get_state(self.steps[current].get_step_index() + 1);

            trace!("Trying update current: {:?}, next: {:?}", self.steps[current].get_step_index(), self.steps[next].get_step_index());

            let mut was_updated = false;
            if !self.steps[next].is_state_final() {
                let next_state = self.steps[current].calculate_next_state();
                if next_state.is_some() {
                    self.steps[next].set_calculated_state(next_state.unwrap());

                    if self.steps[current].is_complete() && self.steps[current].get_input_count() == player_count {
                        self.steps[next].mark_as_complete();
                    }

                }
            } else {
                self.steps[current].mark_as_calculation_not_needed();
            }

            if current == 0 && self.steps[current].get_step_index() < self.drop_steps_before {
                self.steps[next].mark_as_complete();
                let dropped = self.steps.pop_front().unwrap();
                trace!("Dropped step: {:?}", dropped.get_step_index());
            } else {
                current = current + 1;
            }

            self.send_messages(current);
        }

        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Sender<Manager<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

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

    pub fn drop_steps_before(&self, step :usize) {
        self.send(move |manager|{
            manager.drop_steps_before(step);
        }).unwrap();
    }

    pub fn set_requested_step(&self, step: usize) {
        self.send(move |manager|{
            manager.set_requested_step(step);
        });
    }
}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<Manager<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |manager|{
            let step = manager.get_state(input_message.get_step());
            step.set_input(input_message);
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<StateMessage<StateType>> for Sender<Manager<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    fn accept(&self, mut state_message: StateMessage<StateType>) {
        self.send(move |manager|{
            manager.handle_state_message(state_message);
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<InitialInformation<StateType>> for Sender<Manager<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    fn accept(&self, initial_information: InitialInformation<StateType>) {
        self.send(move |manager|{
            manager.player_count = Some(initial_information.get_player_count());
            manager.handle_state_message(StateMessage::new(0, initial_information.get_state()));
        }).unwrap();
    }
}