use commons::time::TimeValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingRequest {
    client_send_time: TimeValue
}

pub struct PingResponse {
    request: PingRequest,
    server_receive_time: TimeValue,
    server_send_time: TimeValue,
}