use commons::time::{
    TimeDuration,
    TimeValue,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::game_time::StartTime;

/// Clients ping the server to measure the round trip time of UDP packets.  This
/// measure is needed to correctly determine the offset between the server's clock
/// and the client's clock.  [`PingRequest`] is a client's request to the server
/// for a [`PingResponse`].
#[derive(Serialize, Deserialize, Debug)]
pub struct PingRequest {
    player_index: usize,
    client_time_sent: TimeValue,
}

impl PingRequest {
    pub fn new(player_index: usize, client_time_sent: TimeValue) -> Self {
        Self {
            player_index,
            client_time_sent,
        }
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }
}

/// The server's response to a [`PingRequest`]
#[derive(Serialize, Deserialize, Debug)]
pub struct PingResponse {
    request: PingRequest,
    server_time_received: TimeValue,
    server_time_sent: TimeValue,
}

impl PingResponse {
    pub fn new(
        request: PingRequest,
        server_time_received: TimeValue,
        server_time_sent: TimeValue,
    ) -> Self {
        Self {
            request,
            server_time_received,
            server_time_sent,
        }
    }
}

/// When a client receives a [`PingResponse`], it creates a [`CompletedPing`]
/// which includes the local time the response was received.
pub struct CompletedPing {
    response: PingResponse,
    client_time_received: TimeValue,
}

impl CompletedPing {
    /// Creates a new [`CompletedPing`]
    pub fn new(response: PingResponse, client_time_received: TimeValue) -> Self {
        Self {
            response,
            client_time_received,
        }
    }

    /// Creates a CompletedPing with zero duration round trip.  This is used for starting the server's clock
    pub fn zero_time_ping() -> Self {
        let zero = TimeValue::from_secs_f64(0.0);
        Self {
            response: PingResponse {
                request: PingRequest {
                    player_index: 0,
                    client_time_sent: zero,
                },
                server_time_received: zero,
                server_time_sent: zero,
            },
            client_time_received: zero,
        }
    }

    /// Calculates the offset between the client and server's clock in seconds.  `offset = clients_time - servers_time`
    pub(super) fn get_remote_to_local_clock_offset(&self) -> f64 {
        //The time the ping spent in queues on the server.  This duration is not part of the network round trip time.
        let server_queue_duration = self
            .response
            .server_time_sent
            .duration_since(&self.response.server_time_received);

        let total_round_trip_duration = self
            .client_time_received
            .duration_since(&self.response.request.client_time_sent);

        let network_round_trip_duration = &total_round_trip_duration - &server_queue_duration;

        let latency_duration = network_round_trip_duration.div_f64(2.0);

        let server_time_at_client_receive = self.response.server_time_sent + latency_duration;

        let offset = self
            .client_time_received
            .duration_since(&server_time_at_client_receive);

        return offset.as_secs_f64();
    }

    /// Converts the server's [`StartTime`] into the client's by adding the offset to the client's [`StartTime`].
    pub(super) fn get_local_start_time(offset: f64, remote_start_time: &StartTime) -> StartTime {
        StartTime::new(remote_start_time.get_time_value() + TimeDuration::from_secs_f64(offset))
    }
}
