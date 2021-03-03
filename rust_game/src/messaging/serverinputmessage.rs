use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInputMessage<ServerInputType>
    where ServerInputType: Clone {

    step: usize,
    server_input: ServerInputType,
}

impl<ServerInputType> ServerInputMessage<ServerInputType>
    where ServerInputType: Clone {

    pub fn new(step: usize, server_input: ServerInputType) -> Self {
        Self{ step, server_input }
    }

    pub fn get_server_input(self) -> ServerInputType {
        self.server_input
    }

    pub fn get_step(&self) -> usize {
        self.step
    }
}

impl<ServerInputType> Clone for ServerInputMessage<ServerInputType>
    where ServerInputType: Clone {

    fn clone(&self) -> Self {
        Self{
            step: self.step,
            server_input: self.server_input.clone(),
        }
    }
}