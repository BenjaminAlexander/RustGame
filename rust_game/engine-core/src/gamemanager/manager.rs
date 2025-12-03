use crate::Input;
use crate::game_time::FrameIndex;
use crate::gamemanager::frame::Frame;
use crate::gamemanager::ManagerObserverTrait;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::{
    StateMessage,
};
use commons::real_time::{
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
};
use log::{trace, warn};
use std::collections::vec_deque::VecDeque;

pub enum ManagerEvent<Game: GameTrait> {
    AdvanceFrameIndex(FrameIndex),
    InputEvent{
        frame_index: FrameIndex,
        player_index: usize,
        input: Game::ClientInput,
        is_authoritative: bool
    },
    AuthoritativeMissingInputEvent{
        frame_index: FrameIndex,
        player_index: usize,
    },
    StateEvent(StateMessage<Game>),
}

pub struct Manager<ManagerObserver: ManagerObserverTrait> {
    current_frame_index: FrameIndex,
    initial_information: InitialInformation<ManagerObserver::Game>,
    //New states at the back, old at the front (index 0)
    steps: VecDeque<Frame<ManagerObserver::Game>>,
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
        manager.on_state_message(message);

        return manager;
    }

    fn get_frame(&mut self, frame_index: FrameIndex) -> Option<&mut Frame<ManagerObserver::Game>> {

        let first_frame = match self.steps.get(0) {
            Some(frame) => frame,
            None => {
                let step = Frame::blank(frame_index, self.initial_information.get_player_count());
                self.steps.push_back(step);
                &self.steps[0]
            },
        };

        if frame_index < first_frame.get_step_index() {
            return None;
        }

        let index_to_get = frame_index.usize() - first_frame.get_step_index().usize();

        while self.steps.len() <= index_to_get {
            self.steps.push_back(Frame::blank(
                self.steps[self.steps.len() - 1].get_step_index() + 1,
                self.initial_information.get_player_count(),
            ));
        }
        return Some(&mut self.steps[index_to_get]);
    }

    fn advance_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        trace!("Setting current frame index: {:?}", frame_index);
        self.current_frame_index = frame_index;
        return EventHandleResult::TryForNextEvent;
    }

    fn on_none_pending(&mut self) -> EventHandleResult {
        // Expand Frame queue to hold up current + 1
        self.get_frame(self.current_frame_index.next());

        let last_open_frame_index = self.initial_information
            .get_server_config()
            .get_last_open_frame_index(self.current_frame_index);

        let mut index: usize = 0;

        while index < self.steps.len() - 1 {
            let frame = &mut self.steps[index];

            if ManagerObserver::IS_SERVER && frame.get_step_index() < last_open_frame_index {
                frame.timeout_remaining_inputs(&self.manager_observer);
            }

            if let Some((state, is_authoritative)) = frame.calculate_next_state(&self.initial_information) {

                self.manager_observer.on_step_message(
                    is_authoritative, 
                    StateMessage::new( frame.get_step_index().next(), state.clone())
                );

                let next_frame_index = {
                    let next_frame = &mut self.steps[index + 1];
                    next_frame.set_state(state, is_authoritative);
                    next_frame.get_step_index()
                };

                if is_authoritative {
                    self.drop_all_frames_before(next_frame_index);
                    index = 0;
                    continue;
                }
            }
            index = index + 1;
        }
        
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_input_message(
        &mut self,
        frame_index: FrameIndex,
        player_index: usize, 
        input: <<ManagerObserver as ManagerObserverTrait>::Game as GameTrait>::ClientInput,
        is_authoritative: bool
    ) -> EventHandleResult {
        if let Some(step) = self.get_frame(frame_index) {
            let input = match is_authoritative {
                true => Input::Authoritative(input),
                false => Input::NonAuthoritative(input),
            };
            step.set_input(player_index, input);
        }
        return EventHandleResult::TryForNextEvent;
    }

    fn on_authoritative_missing_input_message(
        &mut self,
        frame_index: FrameIndex,
        player_index: usize, 
    ) -> EventHandleResult {
        if ManagerObserver::IS_SERVER {
            warn!("The server received an authoritative missing message, ignoring it");
        } else if let Some(step) = self.get_frame(frame_index) {
            step.set_input(player_index, Input::AuthoritativeMissing);
        }
        return EventHandleResult::TryForNextEvent;
    }

    //TODO: there is the potential to maybe drop all frames before the frame that this state is from
    fn on_state_message(
        &mut self,
        state_message: StateMessage<ManagerObserver::Game>,
    ) -> EventHandleResult {

        if let Some(step) = self.get_frame(state_message.get_frame_index()) {
            step.set_state(state_message.take_state(), true);
            //TODO: I think this should work but it causes funky rubberbanding
            //Oh, duh, its because the main loop does stuff like call the observer.  This causes some stuff to not be sent to the observer
            //Not that I think of it, there is the potential for inputs/states to the oldest frame to not go out the observer
            //let frame_index = step.get_step_index();
            //self.drop_all_frames_before(frame_index);
        }
        return EventHandleResult::TryForNextEvent;
    }

    fn drop_all_frames_before(&mut self, frame_index: FrameIndex) {
        while self.steps[0].get_step_index() < frame_index {
            self.steps.pop_front().unwrap();
        }
    }
}

impl<ManagerObserver: ManagerObserverTrait> HandleEvent for Manager<ManagerObserver> {
    type Event = ManagerEvent<ManagerObserver::Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            ManagerEvent::AdvanceFrameIndex(frame_index) => self.advance_frame_index(frame_index),
            ManagerEvent::InputEvent { frame_index, player_index, input, is_authoritative } => self.on_input_message(frame_index, player_index, input, is_authoritative),
            ManagerEvent::AuthoritativeMissingInputEvent { frame_index, player_index } => self.on_authoritative_missing_input_message(frame_index, player_index),
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
