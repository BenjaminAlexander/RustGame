use log::{warn, trace, info};
use crate::messaging::{StateMessage, InputMessage, InitialInformation, ServerInputMessage};
use std::collections::VecDeque;
use crate::interface::{Input, State, InputEvent, StateUpdate, ServerInput, UpdateArg};
use crate::threading::{ConsumerList, ChannelDrivenThread, Sender, Consumer};
use crate::gamemanager::step::Step;
use crate::gamemanager::stepmessage::StepMessage;
use crate::gametime::{TimeMessage, TimeReceived, TimeDuration, TimeValue};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct Manager<StateType, InputType, ServerInputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    is_server: bool,
    drop_steps_before: usize,
    //TODO: send requested state immediately if available
    requested_step: usize,
    initial_information: Option<Arc<InitialInformation<StateType>>>,
    //New states at the back, old at the front (index 0)
    steps: VecDeque<Step<StateType, InputType, ServerInputType, StateUpdateType>>,
    requested_step_consumer_list: ConsumerList<StepMessage<StateType>>,
    completed_step_consumer_list: ConsumerList<StateMessage<StateType>>,
    server_input_consumer_list: ConsumerList<ServerInputMessage<ServerInputType>>,
    grace_period: TimeDuration,
    phantom: PhantomData<(StateUpdateType, ServerInputType)>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
}

impl<StateType, InputType, ServerInputType, StateUpdateType> Manager<StateType, InputType, ServerInputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    pub fn new(is_server: bool, grace_period: TimeDuration) -> Self {
        Self{
            is_server,
            initial_information: None,
            steps: VecDeque::new(),
            requested_step: 0,
            drop_steps_before: 0,
            requested_step_consumer_list: ConsumerList::new(),
            completed_step_consumer_list: ConsumerList::new(),
            server_input_consumer_list: ConsumerList::new(),
            grace_period,
            phantom: PhantomData,

            //metrics
            time_of_last_state_receive: TimeValue::now(),
            time_of_last_input_receive: TimeValue::now(),
        }
    }

    fn get_state(&mut self, step_index: usize) -> Option<&mut Step<StateType, InputType, ServerInputType, StateUpdateType>> {

        if self.initial_information.is_none() {
            return None;
        };

        let intial_information_rc = self.initial_information.as_ref().unwrap().clone();

        if self.steps.is_empty() {
            let step = Step::blank(step_index, intial_information_rc);
            self.steps.push_back(step);
            return Some(&mut self.steps[0]);

        } else if step_index <= self.steps[0].get_step_index() {
            loop {
                let zero_index = self.steps[0].get_step_index();
                if zero_index == step_index {
                    return Some(&mut self.steps[0]);
                } else {
                    self.steps.push_front(Step::blank(zero_index - 1, intial_information_rc.clone()))
                }
            }
        } else {
            let index_to_get = step_index - self.steps[0].get_step_index();
            while self.steps.len() <= index_to_get {
                self.steps.push_back(Step::blank(self.steps[self.steps.len() - 1].get_step_index() + 1, intial_information_rc.clone()));
            }
            return Some(&mut self.steps[index_to_get]);
        }
    }

    fn handle_state_message(&mut self, state_message: StateMessage<StateType>) {
        if let Some(step) = self.get_state(state_message.get_sequence()) {
            step.set_final_state(state_message);
        }
    }

    fn drop_steps_before(&mut self, step :usize) {
        trace!("Setting drop_steps_before: {:?}", step);
        self.drop_steps_before = step;
        if self.requested_step < self.drop_steps_before {
            warn!("Requested step is earlier than drop step: {:?}", self.drop_steps_before);
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

        if self.is_server {
            if let Some(message) = self.steps[step_index].get_server_input_message() {
                self.server_input_consumer_list.accept(&message);
            }
        }
    }
}

impl<StateType, InputType, ServerInputType, StateUpdateType> ChannelDrivenThread<()> for Manager<StateType, InputType, ServerInputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    fn on_none_pending(&mut self) -> Option<()> {

        let now = TimeValue::now();
        let duration_since_last_state = now.duration_since(self.time_of_last_state_receive);
        if duration_since_last_state > TimeDuration::one_second() {
            //warn!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
            //      duration_since_last_state, now, self.time_of_last_state_receive);
        }

        if self.steps.is_empty() {
            trace!("Steps is empty");
            return None;
        }

        if self.initial_information.is_none() {
            return None;
        }

        let mut current: usize = 0;

        self.send_messages(current);

        while self.steps[current].are_inputs_complete() ||
                self.steps[current].get_step_index() < self.requested_step {

            let next = current + 1;
            let should_drop_current = current == 0 && self.steps[current].get_step_index() < self.drop_steps_before;

            self.get_state(self.steps[current].get_step_index() + 1);

            trace!("Trying update current: {:?}, next: {:?}", self.steps[current].get_step_index(), self.steps[next].get_step_index());

            if (self.is_server || !self.steps[next].is_state_deserialized()) &&
                (self.steps[current].need_to_compute_next_state() ||
                (should_drop_current && self.steps[next].is_state_none())) {

                if self.is_server {
                    self.steps[current].calculate_server_input();
                }

                let next_state = self.steps[current].calculate_next_state();
                self.steps[next].set_calculated_state(next_state);
            }

            self.steps[current].mark_as_calculation_not_needed();

            if self.steps[current].are_inputs_complete() {
                self.steps[next].mark_as_complete();
            }

            self.send_messages(current);

            if should_drop_current {

                self.steps[next].mark_as_complete();

                let dropped = self.steps.pop_front().unwrap();
                trace!("Dropped step: {:?}", dropped.get_step_index());
            } else {
                current = current + 1;
            }
        }

        self.send_messages(current);

        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType, ServerInputType, StateUpdateType> Sender<Manager<StateType, InputType, ServerInputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    pub fn add_requested_step_consumer<T>(&self, consumer: T)
        where T: Consumer<StepMessage<StateType>> {

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

    pub fn add_server_input_consumer<T>(&self, consumer: T)
        where T: Consumer<ServerInputMessage<ServerInputType>> {
        self.send(move |manager|{
            manager.server_input_consumer_list.add_consumer(consumer);
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

impl<StateType, InputType, ServerInputType, StateUpdateType> Consumer<InputMessage<InputType>> for Sender<Manager<StateType, InputType, ServerInputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |manager|{
            if let Some(step) = manager.get_state(input_message.get_step()) {
                step.set_input(input_message);
                manager.time_of_last_input_receive = TimeValue::now();
            }
        }).unwrap();
    }
}

impl<StateType, InputType, ServerInputType, StateUpdateType> Consumer<ServerInputMessage<ServerInputType>> for Sender<Manager<StateType, InputType, ServerInputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    fn accept(&self, server_input_message: ServerInputMessage<ServerInputType>) {
        self.send(move |manager|{

            //info!("Server Input received: {:?}", server_input_message.get_step());
            if let Some(step) = manager.get_state(server_input_message.get_step()) {
                step.set_server_input(server_input_message.get_server_input());
            }
        }).unwrap();
    }
}

impl<StateType, InputType, ServerInputType, StateUpdateType> Consumer<StateMessage<StateType>> for Sender<Manager<StateType, InputType, ServerInputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    fn accept(&self, mut state_message: StateMessage<StateType>) {
        self.send(move |manager|{
            manager.handle_state_message(state_message);

            manager.time_of_last_state_receive = TimeValue::now();

        }).unwrap();
    }
}

impl<StateType, InputType, ServerInputType, StateUpdateType> Consumer<InitialInformation<StateType>> for Sender<Manager<StateType, InputType, ServerInputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          ServerInputType: ServerInput,
          StateUpdateType: StateUpdate<StateType, InputType, ServerInputType> {

    fn accept(&self, initial_information: InitialInformation<StateType>) {
        self.send(move |manager|{
            manager.initial_information = Some(Arc::new(initial_information));
            let state = manager.initial_information.as_ref().unwrap().get_state().clone();
            manager.handle_state_message(StateMessage::new(
                0,
                state
            ));
        }).unwrap();
    }
}