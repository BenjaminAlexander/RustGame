use std::sync::mpsc::TryRecvError;

use crate::game_time::StartTime;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::FrameIndexAndState;
use commons::real_time::{
    Factory,
    Receiver,
    Sender,
    TimeSource,
};
use commons::utils::unit_error;
use log::info;

enum Message<Game: GameTrait> {
    InitialInformation(InitialInformation<Game>),
    State(FrameIndexAndState<Game>),
    StartTime(StartTime),
}

/// An error describing why a [CurrentStates] is not available
pub enum StateReceiverError {
    /// The channel has been disconnected and no new states will be received.  
    /// This will occur when the game stops.
    Disconnected,

    /// The channel is connect but not all necessary information or a state has
    /// been received.  This can occur breifly immediatly after the game starts.
    StateNoYetAvailable,
}

/// A struct containing the most recent states calculated by the game.
pub struct CurrentStates<'a, Game: GameTrait> {
    pub time_source: &'a TimeSource,
    pub start_time: &'a StartTime,
    pub initial_information: &'a InitialInformation<Game>,

    /// This is the second newest state that has been received from the game.
    /// Typically, the time of occurance of this state will be slightly in the
    /// past from now.
    pub current_frame: &'a FrameIndexAndState<Game>,

    /// This is the newest state that has been received from the game.  Typically,
    /// the time of occurance of this state will be slightly in the future from
    /// now.
    pub next_frame: &'a FrameIndexAndState<Game>,
}

/// The [StateReceiver] is the receiver for states produced by the game.  The
/// [StateReceiver] receiveds new states as they are calculated and tracks the
/// two newest states.  This typically means that the [StateReceiver] has one
/// state occuring slightly in the past and one state occuring slightly in the
/// future.
pub struct StateReceiver<Game: GameTrait> {
    time_source: TimeSource,
    receiver: Receiver<Message<Game>>,
    start_time: Option<StartTime>,
    current_frame: Option<FrameIndexAndState<Game>>,
    next_frame: Option<FrameIndexAndState<Game>>,
    initial_information: Option<InitialInformation<Game>>,
}

impl<Game: GameTrait> StateReceiver<Game> {
    pub(crate) fn new(factory: &Factory) -> (StateSender<Game>, Self) {
        let (sender, receiver) = factory.new_channel();

        let receiver = Self {
            time_source: factory.get_time_source().clone(),
            receiver,
            start_time: None,
            current_frame: None,
            next_frame: None,
            initial_information: None,
        };

        let sender = StateSender { sender };

        return (sender, receiver);
    }

    pub fn get_step_message(&mut self) -> Result<CurrentStates<Game>, StateReceiverError> {
        loop {
            match self.receiver.try_recv() {
                Ok(Message::InitialInformation(initial_information)) => {
                    self.on_initial_information(initial_information)
                }
                Ok(Message::State(step_message)) => self.on_step_message(step_message),
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
            None => return Err(StateReceiverError::StateNoYetAvailable),
        };

        let initial_information = match &self.initial_information {
            Some(initial_information) => initial_information,
            None => return Err(StateReceiverError::StateNoYetAvailable),
        };

        let next_frame = match &self.next_frame {
            Some(second_step) => second_step,
            None => return Err(StateReceiverError::StateNoYetAvailable),
        };

        let current_frame = match &self.current_frame {
            Some(first_step) => first_step,
            None => next_frame,
        };

        return Ok(CurrentStates {
            time_source: &self.time_source,
            start_time,
            initial_information,
            current_frame,
            next_frame,
        });
        /*




        na1                   na2
                last render
        nb1                   nb2



                let now = self.time_source.now();
                //let latest_time_message = self.data.latest_time_message.as_ref().unwrap();
                //let mut duration_since_start = latest_time_message.get_duration_since_start(now);
                //used to be floor
                //let now_as_fractional_step_index = latest_time_message.get_step_from_actual_time(now);

                //TODO: should probably be interpolating between the current frame (nearest past), to next (nearest future)
                let desired_first_step_index = start_time.get_frame_index(
                    initial_information.get_server_config().get_frame_duration(),
                    &now,
                );

                if first_step.get_frame_index() + 1 != second_step.get_frame_index() {
                    warn!(
                        "Interpolating from non-sequential steps: {:?}, {:?}",
                        first_step.get_frame_index(),
                        second_step.get_frame_index()
                    );
                }

                if (desired_first_step_index.usize() as i64 - first_step.get_frame_index().usize() as i64)
                    .abs()
                    > 1
                {
                    warn!(
                        "Needed step: {:?}, Gotten step: {:?}",
                        desired_first_step_index,
                        first_step.get_frame_index()
                    );
                }

                let mut weight = if second_step.get_frame_index() == first_step.get_frame_index() {
                    1 as f64
                } else {
                    let fractional_frame_index = start_time.get_fractional_frame_index(
                        initial_information.get_server_config().get_frame_duration(),
                        &now,
                    );
                    (fractional_frame_index - first_step.get_frame_index().usize() as f64)
                        / ((second_step.get_frame_index().usize() - first_step.get_frame_index().usize())
                            as f64)
                };

                let interpolate = true;
                if !interpolate {
                    weight = 1 as f64;
                }

                //TODO: this duration since start thing seems strange
                // The render receiver should probably just expose the start time
                let duration_since_start = now.duration_since(start_time.get_time_value());

                let arg = InterpolationArg::new(weight, duration_since_start);
                let interpolation_result = Game::interpolate(
                    self.data.initial_information.as_ref().unwrap(),
                    first_step.get_state(),
                    second_step.get_state(),
                    &arg,
                );

                return Some((duration_since_start, interpolation_result));
                */
    }

    fn on_initial_information(&mut self, initial_information: InitialInformation<Game>) {
        self.initial_information = Some(initial_information);
    }

    fn on_step_message(&mut self, frame_index_and_state: FrameIndexAndState<Game>) {
        let next_frame = match &mut self.next_frame {
            Some(next_frame) => next_frame,
            None => {
                self.next_frame = Some(frame_index_and_state);
                return;
            }
        };

        if next_frame.get_frame_index() == frame_index_and_state.get_frame_index() {
            *next_frame = frame_index_and_state;
            return;
        }

        if next_frame.get_frame_index() < frame_index_and_state.get_frame_index() {
            self.current_frame = self.next_frame.take();
            self.next_frame = Some(frame_index_and_state);
            return;
        }

        let current_frame = match &mut self.current_frame {
            Some(current_frame) => current_frame,
            None => {
                self.current_frame = Some(frame_index_and_state);
                return;
            }
        };

        if current_frame.get_frame_index() <= frame_index_and_state.get_frame_index() {
            *current_frame = frame_index_and_state;
        }
    }

    fn on_start_time(&mut self, start_time: StartTime) {
        self.start_time = Some(start_time);
    }
}

#[derive(Clone)]
pub(crate) struct StateSender<Game: GameTrait> {
    sender: Sender<Message<Game>>,
}

impl<Game: GameTrait> StateSender<Game> {
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
