use commons::time::TimeValue;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingRequest {
    player_index: usize,
    client_send_time: TimeValue,
}

impl PingRequest {
    pub fn new(player_index: usize, client_send_time: TimeValue) -> Self {
        Self {
            player_index,
            client_send_time
        }
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingResponse {
    request: PingRequest,
    server_time_received: TimeValue,
    server_time_sent: TimeValue,
}

impl PingResponse {
    pub fn new (request: PingRequest, server_time_received: TimeValue, server_time_sent: TimeValue) -> Self {
        Self {
            request, 
            server_time_received,
            server_time_sent
        }
    }
}

pub struct CompletedPing {
    response: PingResponse,
    client_time_received: TimeValue
}

impl CompletedPing {
    pub fn new (response: PingResponse, client_time_received: TimeValue) -> Self {
        Self {
            response, 
            client_time_received
        }
    }
}