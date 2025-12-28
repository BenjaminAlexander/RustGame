use {
    crate::{
        FrameIndex,
        Play,
        frame_manager::{
            InputCoordinates,
            ObserveFrames,
            frame::{
                ClientProvidedInput,
                Frame,
                PresentInput,
            },
        },
        interface::InitialInformation,
    },
    commons::{
        real_time::{
            EventHandleResult,
            EventHandlerBuilder,
            EventSender,
            Factory,
            HandleEvent,
            ReceiveMetaData,
        },
        utils::{
            log_error,
            unit_error,
        },
    },
    log::error,
    std::{
        collections::vec_deque::VecDeque,
        ops::ControlFlow,
    },
};

/// The [FrameManager] manages [Frames](Frame) and calculates new
/// [states](Play::State) from [Inputs](Play::InputAggregate) in another thread.
///
/// The [FrameManager] inserts [Inputs](Play::InputAggregate) from local and remote clients as
/// well as authoritative [States](Play::State) from the server into a vector
/// of [Frames](Frame).  When there are no queued events, the [FrameManager] will
/// iterate over the Frames, calculating new States, up to the current
/// [FrameIndex] + 1.
///
/// New States and Inputs which are declared authoritatively missing can be observerd
/// by implementing [ObserveFrames].
#[derive(Clone)]
pub struct FrameManager<Game: Play> {
    sender: EventSender<Event<Game>>,
}

impl<Game: Play> FrameManager<Game> {
    /// Starts a new [FrameManager]
    pub fn new<T: ObserveFrames<Game = Game>>(
        factory: &Factory,
        manager_observer: T,
        initial_information: InitialInformation<T::Game>,
    ) -> Result<Self, ()> {
        let event_handler = EventHandler::new(manager_observer, initial_information)?;

        let thread_name = if T::IS_SERVER {
            "ServerManager"
        } else {
            "ClientManager"
        };

        let sender =
            EventHandlerBuilder::new_thread(factory, thread_name.to_string(), event_handler)
                .map_err(log_error)?;

        Ok(Self { sender })
    }

    /// Advances the current [FrameIndex] of the [FrameManager] on the server.  
    /// The [FrameManager] will compute frames up to `frame_index + 1`.
    pub fn server_advance_frame_index(&self, frame_index: FrameIndex) -> Result<(), ()> {
        let event = Event::ServerAdvanceFrameIndex(frame_index);
        self.sender.send_event(event).map_err(unit_error)
    }

    /// Advances the current [FrameIndex] of the [FrameManager] on the client.  
    /// The [FrameManager] will compute frames up to `frame_index + 1`.
    ///
    /// This will notify the observer of the final completed [Play::InputEvent]
    /// from last current frame index for the local player.  It will also insert a peak
    /// (non-final) input for the local playyer into the new current frame.
    pub fn client_advance_frame_index(
        &self,
        current_coordinates: InputCoordinates,
        peak_of_current_frame_input: Game::InputAggregate,
    ) -> Result<(), ()> {
        let event = Event::ClientAdvanceFrameIndex {
            current_coordinates,
            peak_of_current_frame_input,
        };
        self.sender.send_event(event).map_err(unit_error)
    }

    /// Inserts a [PresentInput] with a [Play::InputAggregate] into the [Frame] at
    /// [InputCoordinates].  If the [FrameIndex] is too far in the past, it input will
    /// be ignored.
    pub fn insert_input(
        &self,
        coordinates: InputCoordinates,
        input: PresentInput<Game::InputAggregate>,
    ) -> Result<(), ()> {
        let event = Event::Input { coordinates, input };
        self.sender.send_event(event).map_err(unit_error)
    }

    /// Inserts a [State](Play::State) into the [Frame] at [FrameIndex].  
    /// If the [FrameIndex] is too far in the past, it state will be ignored.
    pub fn insert_state(&self, frame_index: FrameIndex, state: Game::State) -> Result<(), ()> {
        let event = Event::State { frame_index, state };

        self.sender.send_event(event).map_err(unit_error)
    }

    /// Stops the [FrameManager's][FrameManager] thread which calculates new States
    pub fn stop(&self) -> Result<(), ()> {
        self.sender.send_stop_thread()
    }
}

enum Event<Game: Play> {
    ServerAdvanceFrameIndex(FrameIndex),
    ClientAdvanceFrameIndex {
        current_coordinates: InputCoordinates,
        peak_of_current_frame_input: Game::InputAggregate,
    },
    Input {
        coordinates: InputCoordinates,
        input: PresentInput<Game::InputAggregate>,
    },
    State {
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
    ) -> Result<Self, ()> {
        let state = initial_information.get_initial_state().clone();

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
        let result = frame.set_state(state, true, &manager.manager_observer);

        match result {
            ControlFlow::Continue(()) => Ok(manager),
            ControlFlow::Break(()) => Err(()),
        }
    }

    fn get_frame_queue_index(&mut self, frame_index: FrameIndex) -> Option<usize> {
        let first_frame = match self.frames.get(0) {
            Some(frame) => frame,
            None => {
                let step = Frame::blank(frame_index, self.initial_information.get_player_count());
                self.frames.push_back(step);
                &self.frames[0]
            }
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

    fn on_none_pending(&mut self) -> EventHandleResult {
        // Expand Frame queue to hold up current + 1
        self.get_frame_queue_index(self.current_frame_index.next());

        let mut index = 0;

        while index < self.frames.len() - 1 {
            let frame = &mut self.frames[index];

            if let Some((state, is_authoritative)) =
                frame.calculate_next_state(&self.initial_information)
            {
                let next_frame_index = {
                    let next_frame = &mut self.frames[index + 1];

                    let result =
                        next_frame.set_state(state, is_authoritative, &self.manager_observer);

                    if result.is_break() {
                        return EventHandleResult::StopThread;
                    }

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

    fn set_input(
        &mut self,
        coordinates: &InputCoordinates,
        input: PresentInput<<<ManagerObserver as ObserveFrames>::Game as Play>::InputAggregate>,
    ) {
        #[cfg(debug_assertions)]
        match (&input, ManagerObserver::IS_SERVER) {
            (PresentInput::AuthoritativeMissing, true) => {
                panic!("The server received an authoritative missing message")
            }
            _ => {}
        }

        if let Some(index) = self.get_frame_queue_index(coordinates.get_frame_index()) {
            self.frames[index].set_input(coordinates.get_player_index(), input);
        }
    }

    fn on_state_message(
        &mut self,
        frame_index: FrameIndex,
        state: <<ManagerObserver as ObserveFrames>::Game as Play>::State,
    ) -> ControlFlow<()> {
        #[cfg(debug_assertions)]
        if ManagerObserver::IS_SERVER {
            panic!("Remote states should only be received by the client")
        }

        let index = match self.get_frame_queue_index(frame_index) {
            Some(index) => index,
            None => return ControlFlow::Continue(()),
        };

        let frame = &mut self.frames[index];
        frame.set_state(state, true, &self.manager_observer)?;

        // Drop frames that are no longer needed to calculate updates over to get
        // to an authoritative state.
        if frame_index <= self.current_frame_index {
            self.drop_all_frames_before(frame_index);
        }

        ControlFlow::Continue(())
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
            Event::ServerAdvanceFrameIndex(frame_index) => {
                #[cfg(debug_assertions)]
                if !ManagerObserver::IS_SERVER {
                    panic!("This message is for the server frame manager only");
                }

                self.current_frame_index = frame_index;

                if ManagerObserver::IS_SERVER {
                    let last_open_frame_index = self
                        .initial_information
                        .get_server_config()
                        .get_last_open_frame_index(self.current_frame_index);

                    let mut index = 0;

                    while index < self.frames.len()
                        && self.frames[index].get_frame_index() < last_open_frame_index
                    {
                        if self.frames[index]
                            .close_frame(&self.manager_observer)
                            .is_break()
                        {
                            return EventHandleResult::StopThread;
                        }
                        index += 1;
                    }
                }
            }
            Event::ClientAdvanceFrameIndex {
                current_coordinates,
                peak_of_current_frame_input,
            } => {
                #[cfg(debug_assertions)]
                if ManagerObserver::IS_SERVER {
                    panic!("This message is for the client frame manager only");
                }

                //let previous_coordinates = current_coordinates;

                if let Some(index) = self.get_frame_queue_index(self.current_frame_index) {
                    let previous_input =
                        self.frames[index].get_input(current_coordinates.get_player_index());

                    #[cfg(debug_assertions)]
                    if previous_input.is_authoritative() {
                        panic!("This should never be authoritative");
                    }

                    match previous_input.get_input() {
                        Some(previous_input) => {
                            let previous_coordinates = InputCoordinates::new(
                                self.current_frame_index,
                                current_coordinates.get_player_index(),
                            );
                            let result = self
                                .manager_observer
                                .input_authoritative(previous_coordinates, previous_input.clone());
                            if result.is_break() {
                                return EventHandleResult::StopThread;
                            }
                        }
                        None =>
                        {
                            #[cfg(debug_assertions)]
                            if current_coordinates.get_frame_index() > FrameIndex::zero() {
                                error!("There should always be an input");
                            }
                        }
                    }
                } else {
                    error!("There should always be a previous frame");
                }

                self.set_input(
                    &current_coordinates,
                    PresentInput::ClientProvided(ClientProvidedInput::NonAuthoritative(
                        peak_of_current_frame_input,
                    )),
                );

                self.current_frame_index = current_coordinates.get_frame_index();
            }
            Event::Input { coordinates, input } => self.set_input(&coordinates, input),
            Event::State { frame_index, state } => {
                let result = self.on_state_message(frame_index, state);
                if result.is_break() {
                    return EventHandleResult::StopThread;
                }
            }
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
