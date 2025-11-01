use crate::gamemanager::manager::ManagerEvent::{
    DropStepsBeforeEvent,
    InitialInformationEvent,
    InputEvent,
    ServerInputEvent,
    SetRequestedStepEvent,
    StateEvent,
};
use crate::gamemanager::step::Step;
use crate::gamemanager::ManagerObserverTrait;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::{
    InputMessage,
    ServerInputMessage,
    StateMessage,
};
use commons::factory::FactoryTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use commons::time::{
    TimeDuration,
    TimeValue,
};
use log::{
    trace,
    warn,
};
use std::collections::vec_deque::VecDeque;
use std::sync::Arc;

pub enum ManagerEvent<Game: GameTrait> {
    //TODO: can drop steps and requested Step be combined?
    DropStepsBeforeEvent(usize),
    SetRequestedStepEvent(usize),
    //TODO: get initial information before starting
    InitialInformationEvent(InitialInformation<Game>),
    InputEvent(InputMessage<Game>),
    ServerInputEvent(ServerInputMessage<Game>),
    StateEvent(StateMessage<Game>),
}

pub struct Manager<ManagerObserver: ManagerObserverTrait> {
    factory: ManagerObserver::Factory,
    drop_steps_before: usize,
    //TODO: send requested state immediately if available
    requested_step: usize,
    initial_information: Option<Arc<InitialInformation<ManagerObserver::Game>>>,
    //New states at the back, old at the front (index 0)
    steps: VecDeque<Step<ManagerObserver::Game>>,
    manager_observer: ManagerObserver,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
}

impl<ManagerObserver: ManagerObserverTrait> Manager<ManagerObserver> {
    pub fn new(factory: ManagerObserver::Factory, manager_observer: ManagerObserver) -> Self {
        Self {
            initial_information: None,
            steps: VecDeque::new(),
            requested_step: 0,
            drop_steps_before: 0,
            manager_observer,

            //metrics
            time_of_last_state_receive: factory.get_time_source().now(),
            time_of_last_input_receive: factory.get_time_source().now(),

            factory,
        }
    }

    fn get_state(&mut self, step_index: usize) -> Option<&mut Step<ManagerObserver::Game>> {
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
                    self.steps
                        .push_front(Step::blank(zero_index - 1, intial_information_rc.clone()))
                }
            }
        } else {
            let index_to_get = step_index - self.steps[0].get_step_index();
            while self.steps.len() <= index_to_get {
                self.steps.push_back(Step::blank(
                    self.steps[self.steps.len() - 1].get_step_index() + 1,
                    intial_information_rc.clone(),
                ));
            }
            return Some(&mut self.steps[index_to_get]);
        }
    }

    fn handle_state_message(&mut self, state_message: StateMessage<ManagerObserver::Game>) {
        if let Some(step) = self.get_state(state_message.get_sequence()) {
            step.set_final_state(state_message);
        }
    }

    fn drop_steps_before(mut self, step: usize) -> EventHandleResult<Self> {
        trace!("Setting drop_steps_before: {:?}", step);
        self.drop_steps_before = step;
        if self.requested_step < self.drop_steps_before {
            warn!(
                "Requested step is earlier than drop step: {:?}",
                self.drop_steps_before
            );
            return self.set_requested_step(step);
        }

        return EventHandleResult::TryForNextEvent(self);
    }

    fn set_requested_step(mut self, step: usize) -> EventHandleResult<Self> {
        trace!("Setting requested_step: {:?}", step);
        self.requested_step = step;
        return EventHandleResult::TryForNextEvent(self);
    }

    fn send_messages(&mut self, step_index: usize) {
        let changed_message_option = self.steps[step_index].get_changed_message();
        if changed_message_option.is_some() {
            self.manager_observer
                .on_step_message(changed_message_option.unwrap().clone());
        }

        let complete_message_option = self.steps[step_index].get_complete_message();
        if complete_message_option.is_some() {
            self.manager_observer
                .on_completed_step(complete_message_option.as_ref().unwrap().clone());
        }

        if ManagerObserver::IS_SERVER {
            if let Some(message) = self.steps[step_index].get_server_input_message() {
                self.manager_observer.on_server_input_message(message);
            }
        }
    }

    fn on_none_pending(mut self) -> EventHandleResult<Self> {
        let now = self.factory.get_time_source().now();
        let duration_since_last_state = now.duration_since(&self.time_of_last_state_receive);
        if duration_since_last_state > TimeDuration::ONE_SECOND {
            //warn!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
            //      duration_since_last_state, now, self.time_of_last_state_receive);
        }

        if self.steps.is_empty() {
            trace!("Steps is empty");
            return EventHandleResult::WaitForNextEvent(self);
        }

        if self.initial_information.is_none() {
            return EventHandleResult::WaitForNextEvent(self);
        }

        let mut current: usize = 0;

        self.send_messages(current);

        while self.steps[current].are_inputs_complete()
            || self.steps[current].get_step_index() < self.requested_step
        {
            let next = current + 1;
            let should_drop_current =
                current == 0 && self.steps[current].get_step_index() < self.drop_steps_before;

            self.get_state(self.steps[current].get_step_index() + 1);

            trace!(
                "Trying update current: {:?}, next: {:?}",
                self.steps[current].get_step_index(),
                self.steps[next].get_step_index()
            );

            if (ManagerObserver::IS_SERVER || !self.steps[next].is_state_deserialized())
                && (self.steps[current].need_to_compute_next_state()
                    || (should_drop_current && self.steps[next].is_state_none()))
            {
                if ManagerObserver::IS_SERVER {
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

        return EventHandleResult::WaitForNextEvent(self);
    }

    fn on_initial_information(
        mut self,
        initial_information: InitialInformation<ManagerObserver::Game>,
    ) -> EventHandleResult<Self> {
        //TODO: move Arc outside lambda
        self.initial_information = Some(Arc::new(initial_information));
        let state = self
            .initial_information
            .as_ref()
            .unwrap()
            .get_state()
            .clone();
        self.handle_state_message(StateMessage::new(0, state));
        return EventHandleResult::TryForNextEvent(self);
    }

    fn on_input_message(
        mut self,
        input_message: InputMessage<ManagerObserver::Game>,
    ) -> EventHandleResult<Self> {
        if let Some(step) = self.get_state(input_message.get_step()) {
            step.set_input(input_message);
            self.time_of_last_input_receive = self.factory.get_time_source().now();
        }
        return EventHandleResult::TryForNextEvent(self);
    }

    fn on_server_input_message(
        mut self,
        server_input_message: ServerInputMessage<ManagerObserver::Game>,
    ) -> EventHandleResult<Self> {
        //info!("Server Input received: {:?}", server_input_message.get_step());
        if let Some(step) = self.get_state(server_input_message.get_step()) {
            step.set_server_input(server_input_message.get_server_input());
        }
        return EventHandleResult::TryForNextEvent(self);
    }

    fn on_state_message(
        mut self,
        state_message: StateMessage<ManagerObserver::Game>,
    ) -> EventHandleResult<Self> {
        self.handle_state_message(state_message);

        self.time_of_last_state_receive = self.factory.get_time_source().now();

        return EventHandleResult::TryForNextEvent(self);
    }
}

impl<ManagerObserver: ManagerObserverTrait> EventHandlerTrait for Manager<ManagerObserver> {
    type Event = ManagerEvent<ManagerObserver::Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, DropStepsBeforeEvent(step)) => {
                self.drop_steps_before(step)
            }
            ChannelEvent::ReceivedEvent(_, SetRequestedStepEvent(step)) => {
                self.set_requested_step(step)
            }
            ChannelEvent::ReceivedEvent(_, InitialInformationEvent(initial_information)) => {
                self.on_initial_information(initial_information)
            }
            ChannelEvent::ReceivedEvent(_, InputEvent(input_message)) => {
                self.on_input_message(input_message)
            }
            ChannelEvent::ReceivedEvent(_, ServerInputEvent(server_input_message)) => {
                self.on_server_input_message(server_input_message)
            }
            ChannelEvent::ReceivedEvent(_, StateEvent(state_message)) => {
                self.on_state_message(state_message)
            }
            ChannelEvent::Timeout => self.on_none_pending(),
            ChannelEvent::ChannelEmpty => self.on_none_pending(),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        ()
    }
}
