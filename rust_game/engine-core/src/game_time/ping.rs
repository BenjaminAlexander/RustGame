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
}

pub struct PingResponse {
    request: PingRequest,
    server_receive_time: TimeValue,
    server_send_time: TimeValue,
}
