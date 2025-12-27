use std::sync::mpsc::TryRecvError;

use crate::Play;
use crate::frame_manager::FrameIndexAndState;
use crate::game_time::StartTime;
use crate::interface::InitialInformation;
use commons::real_time::{
    Factory,
    Receiver,
    Sender,
    TimeSource,
};
use commons::utils::unit_error;
use log::info;

enum Message<Game: Play> {
    InitialInformation(InitialInformation<Game>),
    State(FrameIndexAndState<Game>),
    StartTime(StartTime),
}

/// An error describing why a [MostRecentState] is not available
pub enum StateReceiverError {
    /// The channel has been disconnected and no new states will be received.
    /// This will occur when the game stops.
    Disconnected,

    /// The channel is connect but not all necessary information or a state has
    /// been received.  This can occur breifly immediatly after the game starts.
    NotYetAvailable,
}

/// A struct containing the most recent [State][Play::State] calculated by the game.
pub struct MostRecentState<'a, Game: Play> {
    time_source: &'a TimeSource,
    start_time: &'a StartTime,
    initial_information: &'a InitialInformation<Game>,
    next_frame: &'a FrameIndexAndState<Game>,
    is_state_new: bool,
}

impl<'a, Game: Play> MostRecentState<'a, Game> {
    /// Returns the [TimeSource] used by the game
    pub fn get_time_source(&self) -> &'a TimeSource {
        self.time_source
    }

    /// Returns the local [StartTime] of the game
    pub fn get_start_time(&self) -> &'a StartTime {
        self.start_time
    }

    /// Returns the [InitialInformation] used by the game
    pub fn get_initial_information(&self) -> &'a InitialInformation<Game> {
        self.initial_information
    }

    /// Returns the newest [FrameIndex][super::FrameIndex] and [State][Play::State]
    /// that has been received from the game.
    ///
    /// Typically, the time of occurance of this state will be slightly in the
    /// future from now.
    pub fn get_frame_index_and_state(&self) -> &'a FrameIndexAndState<Game> {
        self.next_frame
    }

    /// Returns true if this is the first time this state has been returned by
    /// the [StateReceiver]
    pub fn is_state_new(&self) -> bool {
        self.is_state_new
    }
}

/// A receiver for states produced by the game
///
/// The [StateReceiver] receives new states as they are calculated and tracks
/// the most recent state.  This typically means that the [StateReceiver] has one
/// state occuring slightly in the future.
pub struct StateReceiver<Game: Play> {
    time_source: TimeSource,
    receiver: Receiver<Message<Game>>,
    start_time: Option<StartTime>,
    next_frame: Option<FrameIndexAndState<Game>>,
    initial_information: Option<InitialInformation<Game>>,
}

impl<Game: Play> StateReceiver<Game> {
    pub(crate) fn new(factory: &Factory) -> (StateSender<Game>, Self) {
        let (sender, receiver) = factory.new_channel();

        let receiver = Self {
            time_source: factory.get_time_source().clone(),
            receiver,
            start_time: None,
            next_frame: None,
            initial_information: None,
        };

        let sender = StateSender { sender };

        return (sender, receiver);
    }

    /// Gets the most recent state, if it is available
    pub fn get_most_recent_state(
        &mut self,
    ) -> Result<MostRecentState<'_, Game>, StateReceiverError> {
        let mut is_state_new = false;

        loop {
            match self.receiver.try_recv() {
                Ok(Message::InitialInformation(initial_information)) => {
                    self.on_initial_information(initial_information)
                }
                Ok(Message::State(frame_index_and_state)) => {
                    self.on_frame_index_and_state(frame_index_and_state);
                    is_state_new = true;
                }
                Ok(Message::StartTime(start_time)) => self.on_start_time(start_time),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    info!("Channel disconnected.");
                    return Err(StateReceiverError::Disconnected);
                }
            }
        }

        let start_time = match &self.start_time {
            Some(start_time) => start_time,
            None => return Err(StateReceiverError::NotYetAvailable),
        };

        let initial_information = match &self.initial_information {
            Some(initial_information) => initial_information,
            None => return Err(StateReceiverError::NotYetAvailable),
        };

        let next_frame = match &self.next_frame {
            Some(second_step) => second_step,
            None => return Err(StateReceiverError::NotYetAvailable),
        };

        return Ok(MostRecentState {
            time_source: &self.time_source,
            start_time,
            initial_information,
            next_frame,
            is_state_new,
        });
    }

    fn on_initial_information(&mut self, initial_information: InitialInformation<Game>) {
        self.initial_information = Some(initial_information);
    }

    fn on_frame_index_and_state(&mut self, frame_index_and_state: FrameIndexAndState<Game>) {
        let next_frame = match &mut self.next_frame {
            Some(next_frame) => next_frame,
            None => {
                self.next_frame = Some(frame_index_and_state);
                return;
            }
        };

        if next_frame.get_frame_index() <= frame_index_and_state.get_frame_index() {
            *next_frame = frame_index_and_state;
            return;
        }
    }

    fn on_start_time(&mut self, start_time: StartTime) {
        self.start_time = Some(start_time);
    }
}

#[derive(Clone)]
pub(crate) struct StateSender<Game: Play> {
    sender: Sender<Message<Game>>,
}

impl<Game: Play> StateSender<Game> {
    pub(crate) fn send_initial_information(
        &self,
        initial_information: InitialInformation<Game>,
    ) -> Result<(), ()> {
        self.sender
            .send(Message::InitialInformation(initial_information))
            .map_err(unit_error)
    }

    pub(crate) fn send_state(
        &self,
        frame_index_and_state: FrameIndexAndState<Game>,
    ) -> Result<(), ()> {
        self.sender
            .send(Message::State(frame_index_and_state))
            .map_err(unit_error)
    }

    pub(crate) fn send_start_time(&self, start_time: StartTime) -> Result<(), ()> {
        self.sender
            .send(Message::StartTime(start_time))
            .map_err(unit_error)
    }
}
