use log::warn;
use serde::{Deserialize, Serialize};

use crate::frame_manager::{FrameIndexAndState, InputCoordinates, ObserveFrames};
use crate::interface::{
    GameTrait,
    UpdateArg,
};
use crate::{
    FrameIndex,
    InitialInformation,
};
use std::mem::take;
use std::ops::ControlFlow;

/// A Frame is a [State], set of [Inputs](Input), and some metadata used by the
/// [FrameManager](crate::frame_manager::FrameManager) to calculate new [States](GameTrait::State).
pub struct Frame<Game: GameTrait> {
    frame_index: FrameIndex,
    state: Option<FrameState<Game>>,
    inputs: Vec<Input<Game::InputAggregate>>,
    authoritative_input_count: usize,
    need_to_compute_next_state: bool,
}

/// An enum that describes the provenance of a [GameTrait::InputAggregate]
#[derive(Clone, Debug, Default)]
pub enum Input<T> {
    /// [Input::Pending] signifies that an input from a client isn't yet known but may
    /// become known in the future.  If no [ClientProvidedInput] is received in 
    /// a timely manner, the server will upgrate this to 
    /// [PresentInput::AuthoritativeMissing] and distribute it to the clients.
    #[default]
    Pending,

    /// [Input::Present] signifies that something, even if it is non-authoritative, 
    /// is known about the client's input.
    Present(PresentInput<T>),
}

impl<T> Input<T> {
    /// Returns true if the status of the input is authoritatively known.
    pub fn is_authoritative(&self) -> bool {
        match self {
            Input::Pending => false,
            Input::Present(present) => present.is_authoritative(),
        }
    }

    /// Returns Some [GameTrait::InputAggregate] if one is present
    pub fn input(&self) -> Option<&T> {
        match self {
            Input::Pending => None,
            Input::Present(present) => present.input(),
        }
    }

    pub fn type_as_str(&self) -> &str {
        match self {
            Input::Pending => "Pending",
            Input::Present(present_input) => present_input.type_as_str(),
        }
    }
}

/// An enum representing the different ways a client's input can be known
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PresentInput<T> {

    /// The client has submitted an input.
    ClientProvided(ClientProvidedInput<T>),

    /// The client never submitted an input in a timely manner and the server
    /// has authoritatively decided that the client cannot submit an input in the future
    AuthoritativeMissing,
}

impl<T> PresentInput<T> {
    /// Returns true if the status of the input is authoritatively known.
    pub fn is_authoritative(&self) -> bool {
        match self {
            PresentInput::ClientProvided(client_provided) => client_provided.is_authoritative(),
            PresentInput::AuthoritativeMissing => true,
        }
    }

    pub fn input(&self) -> Option<&T> {
        match self {
            PresentInput::ClientProvided(client_provided) => Some(client_provided.input()),
            PresentInput::AuthoritativeMissing => None,
        }
    }

    pub fn type_as_str(&self) -> &str {
        match self {
            PresentInput::ClientProvided(client_provided) => client_provided.type_as_str(),
            PresentInput::AuthoritativeMissing => "AuthoritativeMissing",
        }
    }
}

/// An enum representing the different ways a client can provide an input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientProvidedInput<T> {

    /// The Input has been received from the client.  This input can be superseded 
    /// by the client submitting another, more current input.  Also, this input 
    /// can still potentially be rejected by the server if the client's message 
    /// is dropped or is late.
    /// 
    /// The client will always insert [ClientProvidedInput::NonAuthoritative] into 
    /// its own [super::FrameManager].  Clients do not know if their inputs will 
    /// make it to the server in time to be accepted.  This is the case even when 
    /// the client has made a [ClientProvidedInput::Authoritative] submission to 
    /// the server.
    /// 
    /// If the server does not receive a [ClientProvidedInput::Authoritative] in time, 
    /// the server will upgrade the last [ClientProvidedInput::NonAuthoritative] it 
    /// received to be authoritative and redistribute it to the clients.
    NonAuthoritative(T),

    /// The Input has been sent from the client to the server and the server has 
    /// accepted it and redistributed it back to the clients.
    /// 
    /// [ClientProvidedInput::Authoritative] inputs are always sent directly to 
    /// the server.  Clients will insert the same [GameTrait::ClientInput] into 
    /// their [super::FrameManager] as a [ClientProvidedInput::NonAuthoritative].
    /// The server will redistribute the authoritative input to all clients if it 
    /// is received in a timely manner.
    Authoritative(T),
}

impl<T> ClientProvidedInput<T> {
    /// Returns true if the status of the input is authoritatively known.
    pub fn is_authoritative(&self) -> bool {
        match self {
            ClientProvidedInput::NonAuthoritative(_) => false,
            ClientProvidedInput::Authoritative(_) => true,
        }
    }

    pub fn input(&self) -> &T {
        match self {
            ClientProvidedInput::NonAuthoritative(input) => input,
            ClientProvidedInput::Authoritative(input) => input,
        }
    }

    pub fn type_as_str(&self) -> &str {
        match self {
            ClientProvidedInput::NonAuthoritative(_) => "NonAuthoritative",
            ClientProvidedInput::Authoritative(_) => "Authoritative",
        }
    }
}

/// An enum describing the provenance of a [GameTrait::State].
#[derive(Debug)]
pub enum FrameState<Game: GameTrait> {

    /// The state has been calculated from another authoritative State and a
    /// complete set of authoritative inputs, or has been received from the
    /// server.
    Authoritative(Game::State),

    /// The state has been calculated from either a non-authoritative State or
    /// at least one non-authoritative input
    NonAuthoritative(Game::State),
}

impl<Game: GameTrait> FrameState<Game> {

    /// Returns true if this state was computed from an previous authoritative state 
    /// and a complete set of authoritative inputs for the clients.
    pub fn is_authoritative(&self) -> bool {
        match self {
            FrameState::Authoritative(_) => true,
            FrameState::NonAuthoritative(_) => false,
        }
    }

    /// Gets the state
    pub fn get_state(&self) -> &Game::State {
        match self {
            FrameState::Authoritative(state) => state,
            FrameState::NonAuthoritative(state) => state,
        }
    }
}

impl<Game: GameTrait> Frame<Game> {
    pub fn blank(step_index: FrameIndex, player_count: usize) -> Self {
        let inputs = vec![Input::Pending; player_count];

        return Self {
            frame_index: step_index,
            state: None,
            inputs,
            authoritative_input_count: 0,
            need_to_compute_next_state: true,
        };
    }

    pub fn set_input(&mut self, player_index: usize, input: PresentInput<Game::InputAggregate>) {  
        
        let current_input = &mut self.inputs[player_index];

        if current_input.is_authoritative() {
            warn!(
                "Received a duplicate input where an authoritative one has already been received, ignorning it. Player: {:?}\n{:?}\nCurrent: {:}\nIgnored: {:?}", 
                player_index,
                self.frame_index,
                current_input.type_as_str(),
                input.type_as_str()
            );
            return;
        }

        //TODO: migrate to 2024 edition and use a chain if let
        if let Input::Present(PresentInput::ClientProvided(_)) = current_input {
            if let PresentInput::AuthoritativeMissing = input {
                warn!("Local input was rejected by the server");
            }
        }

        if input.is_authoritative() {
            self.authoritative_input_count = self.authoritative_input_count + 1;
        }

        *current_input = Input::Present(input);
        self.need_to_compute_next_state = true;
    }

    pub fn get_input(&self, player_index: usize) -> &Input<Game::InputAggregate> {
        &self.inputs[player_index]
    }

    pub fn close_frame(
        &mut self,
        observer: &impl ObserveFrames<Game = Game>,
    ) -> ControlFlow<()> {
        for (player_index, input_enum) in &mut self.inputs.iter_mut().enumerate() {
            match take(input_enum) {
                Input::Pending => {
                    self.authoritative_input_count = self.authoritative_input_count + 1;
                    *input_enum = Input::Present(PresentInput::AuthoritativeMissing);
                    self.need_to_compute_next_state = true;

                    //TODO: Make a way to timeout all of a player's inputs immediatly when they disconnect.
                    observer.input_authoritatively_missing(InputCoordinates::new(self.frame_index, player_index))?;
                },
                Input::Present(PresentInput::ClientProvided(ClientProvidedInput::NonAuthoritative(input))) => {
                    self.authoritative_input_count = self.authoritative_input_count + 1;
                    *input_enum = Input::Present(PresentInput::ClientProvided(ClientProvidedInput::Authoritative(input.clone())));
                    self.need_to_compute_next_state = true;

                    //TODO: Make a way to timeout all of a player's inputs immediatly when they disconnect.
                    observer.input_authoritative(InputCoordinates::new(self.frame_index, player_index), input.clone())?;
                },
                taken_input_enum => *input_enum = taken_input_enum,
            };
        }

        ControlFlow::Continue(())
    }

    pub fn are_inputs_complete(&self) -> bool {
        self.authoritative_input_count == self.inputs.len()
    }

    pub fn set_state(
        &mut self,
        state: Game::State,
        is_authoritative: bool,
        observer: &impl ObserveFrames<Game = Game>,
    ) -> ControlFlow<()> {
        let old_state_is_authoritative = self.is_state_authoritative();

        self.state = Some(if is_authoritative {
            FrameState::Authoritative(state.clone())
        } else {
            FrameState::NonAuthoritative(state.clone())
        });

        if old_state_is_authoritative {
            return ControlFlow::Continue(());
        }

        self.need_to_compute_next_state = true;

        observer.new_state(
            is_authoritative,
            FrameIndexAndState::new(self.frame_index, state),
        )
    }

    pub fn calculate_next_state(
        &mut self,
        initial_information: &InitialInformation<Game>,
    ) -> Option<(Game::State, bool)> {
        if !self.need_to_compute_next_state {
            return None;
        }

        let state = match &self.state {
            Some(state) => state,
            None => {
                panic!("Tried to compute next state from a missing state");
            }
        };

        let is_next_state_authoritative = self.are_inputs_complete() && state.is_authoritative();

        let arg = UpdateArg::new(initial_information, self.frame_index, state, &self.inputs);

        let next_state = Game::get_next_state(&arg);

        self.need_to_compute_next_state = false;

        Some((next_state, is_next_state_authoritative))
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        return self.frame_index;
    }

    pub fn is_state_authoritative(&self) -> bool {
        match self.state {
            None => false,
            Some(FrameState::Authoritative(_)) => true,
            Some(FrameState::NonAuthoritative(_)) => false,
        }
    }
}
