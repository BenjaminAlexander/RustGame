use crate::{FrameIndex, Input};
use crate::frame_manager::frame::Frame;
use crate::frame_manager::ObserveFrames;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use commons::real_time::{
    EventHandleResult, EventHandlerBuilder, EventSender, Factory, HandleEvent, ReceiveMetaData
};
use commons::utils::unit_error;
use std::collections::vec_deque::VecDeque;
use std::io::Error;

/// The [FrameManager] manages [Frames](Frame) and calculates new 
/// [states](GameTrait::State) from [Inputs](Input) in another thread.
/// 
/// The [FrameManager] queues [Inputs](Input) from local and remote clients as well as authoritative [Game] 
#[derive(Clone)]
pub struct FrameManager<Game: GameTrait> {
    sender: EventSender<Event<Game>>
}

impl<Game: GameTrait> FrameManager<Game> {

    /// Starts a new [FrameManager]
    pub fn new<T: ObserveFrames<Game = Game>>(
        factory: &Factory,
        manager_observer: T,
        initial_information: InitialInformation<T::Game>,
    ) -> Result<Self, Error> {

        let event_handler = EventHandler::new(manager_observer, initial_information);

        let thread_name = if T::IS_SERVER {
            "ServerManager"
        } else {
            "ClientManager"
        };

        let sender = EventHandlerBuilder::new_thread(
            factory, 
            thread_name.to_string(), 
            event_handler
        )?;

        Ok(Self { sender })
    }

    /// Advances the current [FrameIndex] of the [FrameManager].  The [FrameManager] will compute frames up to `frame_index + 1`.
    pub fn advance_frame_index(&self, frame_index: FrameIndex) -> Result<(), ()> {
        let event = Event::AdvanceFrameIndex(frame_index);
        self.sender.send_event(event).map_err(unit_error)
    }

    /// Inserts a [Input] with a [GameTrait::ClientInput] into the [Frame] at [FrameIndex].  If the [FrameIndex] is too far in the past, it input will be ignored.
    pub fn insert_input(
        &self, 
        frame_index: FrameIndex,
        player_index: usize,
        input: Game::ClientInput,
        is_authoritative: bool
    ) -> Result<(), ()> {
        let event = Event::Input { 
            frame_index, 
            player_index, 
            input, 
            is_authoritative 
        };

        self.sender.send_event(event).map_err(unit_error)
    }

    /// Inserts an [Input::AuthoritativeMissing] into the [Frame] at [FrameIndex].  If the [FrameIndex] is too far in the past, it input will be ignored.
    pub fn insert_missing_input(
        &self, 
        frame_index: FrameIndex,
        player_index: usize,
    ) -> Result<(), ()> {
        let event = Event::AuthoritativeMissingInput { 
            frame_index, 
            player_index, 
        };

        self.sender.send_event(event).map_err(unit_error)
    }

    /// Inserts a [State](GameTrait::State) into the [Frame] at [FrameIndex].  If the [FrameIndex] is too far in the past, it state will be ignored.
    pub fn insert_state(
        &self, 
        frame_index: FrameIndex,
        state: Game::State,
    ) -> Result<(), ()> {
        let event = Event::State { 
            frame_index, 
            state, 
        };

        self.sender.send_event(event).map_err(unit_error)
    }
}

enum Event<Game: GameTrait> {
    AdvanceFrameIndex(FrameIndex),
    Input{
        frame_index: FrameIndex,
        player_index: usize,
        input: Game::ClientInput,
        is_authoritative: bool
    },
    AuthoritativeMissingInput{
        frame_index: FrameIndex,
        player_index: usize,
    },
    State{
        frame_index: FrameIndex,
        state: Game::State,
    },
}

struct EventHandler<ManagerObserver: ObserveFrames> {
    current_frame_index: FrameIndex,
    initial_information: InitialInformation<ManagerObserver::Game>,
    //New states at the back, old at the front (index 0)
    frames: VecDeque<Frame<ManagerObserver::Game>>,
    manager_observer: ManagerObserver,
}

impl<ManagerObserver: ObserveFrames> EventHandler<ManagerObserver> {
    fn new(
        manager_observer: ManagerObserver,
        initial_information: InitialInformation<ManagerObserver::Game>,
    ) -> Self {
        let state = initial_information.get_state().clone();

        let mut manager = Self {
            current_frame_index: FrameIndex::zero(),
            initial_information,
            frames: VecDeque::new(),
            manager_observer,
        };

        // Set state for FrameIndex 0 and send it as authoritative
        let index = match manager.get_frame_queue_index(FrameIndex::zero()) {
            Some(index) => index,
            None => panic!("Getting the zero frame should never fail"),
        };

        let frame = &mut manager.frames[index];
        frame.set_state(state, true, &manager.manager_observer);

        return manager;
    }

    fn get_frame_queue_index(&mut self, frame_index: FrameIndex) -> Option<usize> {

        let first_frame = match self.frames.get(0) {
            Some(frame) => frame,
            None => {
                let step = Frame::blank(frame_index, self.initial_information.get_player_count());
                self.frames.push_back(step);
                &self.frames[0]
            },
        };

        if frame_index < first_frame.get_frame_index() {
            return None;
        }

        let index_to_get = frame_index.usize() - first_frame.get_frame_index().usize();

        while self.frames.len() <= index_to_get {
            self.frames.push_back(Frame::blank(
                self.frames[self.frames.len() - 1].get_frame_index() + 1,
                self.initial_information.get_player_count(),
            ));
        }
        return Some(index_to_get);
    }

    fn get_frame(&mut self, frame_index: FrameIndex) -> Option<&mut Frame<ManagerObserver::Game>> {
        match self.get_frame_queue_index(frame_index) {
            Some(index) => Some(&mut self.frames[index]),
            None => None,
        }
    }

    fn advance_frame_index(&mut self, frame_index: FrameIndex) {
        self.current_frame_index = frame_index;

        if ManagerObserver::IS_SERVER {
            let last_open_frame_index = self.initial_information
                .get_server_config()
                .get_last_open_frame_index(self.current_frame_index);

            let mut index = 0;

            while index < self.frames.len() && self.frames[index].get_frame_index() < last_open_frame_index {
                self.frames[index].timeout_remaining_inputs(&self.manager_observer);
                index += 1;
            }
        }
    }

    fn on_none_pending(&mut self) -> EventHandleResult {
        // Expand Frame queue to hold up current + 1
        self.get_frame_queue_index(self.current_frame_index.next());

        let mut index = 0;

        while index < self.frames.len() - 1 {
            let frame = &mut self.frames[index];

            if let Some((state, is_authoritative)) = frame.calculate_next_state(&self.initial_information) {

                let next_frame_index = {
                    let next_frame = &mut self.frames[index + 1];
                    next_frame.set_state(state, is_authoritative, &self.manager_observer);
                    next_frame.get_frame_index()
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
        input: <<ManagerObserver as ObserveFrames>::Game as GameTrait>::ClientInput,
        is_authoritative: bool
    ) {
        if let Some(step) = self.get_frame(frame_index) {
            let input = match is_authoritative {
                true => Input::Authoritative(input),
                false => Input::NonAuthoritative(input),
            };
            step.set_input(player_index, input);
        }
    }

    fn on_authoritative_missing_input_message(
        &mut self,
        frame_index: FrameIndex,
        player_index: usize, 
    ) {
        #[cfg(debug_assertions)]
        if ManagerObserver::IS_SERVER {
            panic!("The server received an authoritative missing message")
        }

        if let Some(step) = self.get_frame(frame_index) {
            step.set_input(player_index, Input::AuthoritativeMissing);
        }
    }

    fn on_state_message(
        &mut self,
        frame_index: FrameIndex,
        state: <<ManagerObserver as ObserveFrames>::Game as GameTrait>::State,
    ) {

        #[cfg(debug_assertions)]
        if ManagerObserver::IS_SERVER {
            panic!("Remote states should only be received by the client")
        }

        let index = match self.get_frame_queue_index(frame_index) {
            Some(index) => index,
            None => return,
        };

        let frame = &mut self.frames[index];
        frame.set_state(state, true, &self.manager_observer);

        // Drop frames that are no longer needed to calculate updates over to get
        // to an authoritative state.
        if frame_index <= self.current_frame_index {
            self.drop_all_frames_before(frame_index);
        }
    }

    fn drop_all_frames_before(&mut self, frame_index: FrameIndex) {
        while self.frames[0].get_frame_index() < frame_index {
            self.frames.pop_front().unwrap();
        }
    }
}

impl<ManagerObserver: ObserveFrames> HandleEvent for EventHandler<ManagerObserver> {
    type Event = Event<ManagerObserver::Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            Event::AdvanceFrameIndex(frame_index) => self.advance_frame_index(frame_index),
            Event::Input { frame_index, player_index, input, is_authoritative } => self.on_input_message(frame_index, player_index, input, is_authoritative),
            Event::AuthoritativeMissingInput { frame_index, player_index } => self.on_authoritative_missing_input_message(frame_index, player_index),
            Event::State{ frame_index, state } => self.on_state_message(frame_index, state),
        };

        EventHandleResult::TryForNextEvent
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
