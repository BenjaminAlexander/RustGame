use crate::game_time::FrameIndex;
use crate::gamemanager::step::Step;
use crate::gamemanager::ManagerObserverTrait;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::{
    InputMessage,
    StateMessage,
};
use commons::real_time::{
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
};
use log::{error, trace, warn};
use std::collections::vec_deque::VecDeque;

pub enum ManagerEvent<Game: GameTrait> {
    AdvanceFrameIndex(FrameIndex),
    InputEvent(InputMessage<Game>),
    StateEvent(StateMessage<Game>),
}

pub struct Manager<ManagerObserver: ManagerObserverTrait> {
    current_frame_index: FrameIndex,
    initial_information: InitialInformation<ManagerObserver::Game>,
    //New states at the back, old at the front (index 0)
    steps: VecDeque<Step<ManagerObserver::Game>>,
    manager_observer: ManagerObserver,
}

impl<ManagerObserver: ManagerObserverTrait> Manager<ManagerObserver> {
    pub fn new(
        manager_observer: ManagerObserver,
        initial_information: InitialInformation<ManagerObserver::Game>,
    ) -> Self {
        let state = initial_information.get_state().clone();

        let mut manager = Self {
            current_frame_index: FrameIndex::zero(),
            initial_information,
            steps: VecDeque::new(),
            manager_observer,
        };

        // Set state for FrameIndex 0 and send it as authoritative
        let message = StateMessage::new(FrameIndex::zero(), state);
        manager.manager_observer.on_step_message(true, message.clone());
        manager.handle_state_message(message);

        return manager;
    }

    //TODO: rename step
    fn get_state(&mut self, step_index: FrameIndex) -> Option<&mut Step<ManagerObserver::Game>> {
        if self.steps.is_empty() {
            let step = Step::blank(step_index, self.initial_information.get_player_count());
            self.steps.push_back(step);
            return Some(&mut self.steps[0]);
        } else if step_index < self.steps[0].get_step_index() {
            return None;
        } else {
            let index_to_get = step_index.usize() - self.steps[0].get_step_index().usize();
            while self.steps.len() <= index_to_get {
                self.steps.push_back(Step::blank(
                    self.steps[self.steps.len() - 1].get_step_index() + 1,
                    self.initial_information.get_player_count(),
                ));
            }
            return Some(&mut self.steps[index_to_get]);
        }
    }

    fn handle_state_message(&mut self, state_message: StateMessage<ManagerObserver::Game>) {
        if let Some(step) = self.get_state(state_message.get_frame_index()) {
            step.set_state(state_message.take_state(), true);
        }
    }

    fn advance_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        trace!("Setting current frame index: {:?}", frame_index);
        self.current_frame_index = frame_index;
        return EventHandleResult::TryForNextEvent;
    }

    fn on_none_pending(&mut self) -> EventHandleResult {
        // Expand Frame queue to hold up current + 1
        self.get_state(self.current_frame_index.next());

        let last_open_frame_index = self.initial_information
            .get_server_config()
            .get_last_open_frame_index(self.current_frame_index);

        let mut current: usize = 0;

        trace!("Starting update loop");

        while current < self.steps.len() - 1 {
            let next = current + 1;

            trace!(
                "Trying update current: {:?}, next: {:?}",
                self.steps[current].get_step_index(),
                self.steps[next].get_step_index()
            );

            if ManagerObserver::IS_SERVER && self.steps[current].get_step_index() < last_open_frame_index {
                self.steps[current].timeout_remaining_inputs();
            }

            if let Some((state, is_authoritative)) =
                self.steps[current].calculate_next_state(&self.initial_information) {

                self.manager_observer.on_step_message(
                    is_authoritative, 
                    StateMessage::new( self.steps[current].get_step_index().next(), state.clone())
                );

                let next_frame_index = {
                    let next_frame = &mut self.steps[next];
                    next_frame.set_state(state, is_authoritative);
                    next_frame.get_step_index()
                };

                if is_authoritative {
                    while self.steps[0].get_step_index() < next_frame_index {
                        let dropped = self.steps.pop_front().unwrap();
                        trace!("Dropped {:?} since the following frame's state is authoritative", dropped.get_step_index());
                    }
                    current = 0;
                    continue;
                }
            }
            current = current + 1;
        }
        
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_input_message(
        &mut self,
        input_message: InputMessage<ManagerObserver::Game>,
    ) -> EventHandleResult {
        if let Some(step) = self.get_state(input_message.get_frame_index()) {
            step.set_input(input_message);
        }
        return EventHandleResult::TryForNextEvent;
    }

    //TODO: remove?
    fn on_state_message(
        &mut self,
        state_message: StateMessage<ManagerObserver::Game>,
    ) -> EventHandleResult {
        self.handle_state_message(state_message);
        return EventHandleResult::TryForNextEvent;
    }
}

impl<ManagerObserver: ManagerObserverTrait> HandleEvent for Manager<ManagerObserver> {
    type Event = ManagerEvent<ManagerObserver::Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            ManagerEvent::AdvanceFrameIndex(frame_index) => self.advance_frame_index(frame_index),
            ManagerEvent::InputEvent(input_message) => self.on_input_message(input_message),
            ManagerEvent::StateEvent(state_message) => self.on_state_message(state_message),
        }
    }

    fn on_timeout(&mut self) -> EventHandleResult {
        self.on_none_pending()
    }

    fn on_channel_empty(&mut self) -> EventHandleResult {
        self.on_none_pending()
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
