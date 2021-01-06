use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<StateType, InputType>
    where InputType: Clone,
          StateType: Clone {

    sequence: i32,
    inputs: Vec<Option<InputType>>,
    state: Option<StateType>
}

impl<StateType, InputType> StateMessage<StateType, InputType>
    where InputType: Clone,
          StateType: Clone {

    pub fn new(sequence: i32, inputs: Vec<Option<InputType>>, state: Option<StateType>) -> StateMessage<StateType, InputType> {
        StateMessage{ sequence, inputs, state }
    }

    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }
}

impl<StateType, InputType> Clone for StateMessage<StateType, InputType>
    where InputType: Clone,
          StateType: Clone {
    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            inputs: self.inputs.clone(),
            state: self.state.clone()
        }
    }
}