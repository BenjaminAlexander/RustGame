use serde::{Deserialize, Serialize};
use crate::messaging::StateMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<StateType>
    where StateType: Clone {

    player_count: usize,
    player_index: usize,
    state: StateType,
}

impl<StateType> InitialInformation<StateType>
    where StateType: Clone {

    pub fn new(player_count: usize,
               player_index: usize,
               state: StateType) -> Self {

        Self{ player_count, player_index, state }
    }

    pub fn get_state(self) -> StateType {
        self.state
    }

    pub fn get_player_count(&self) -> usize {
        self.player_count
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }
}

impl<StateType> Clone for InitialInformation<StateType>
    where StateType: Clone {

    fn clone(&self) -> Self {
        Self{
            player_count: self.player_count,
            player_index: self.player_index,
            state: self.state.clone(),
        }
    }
}