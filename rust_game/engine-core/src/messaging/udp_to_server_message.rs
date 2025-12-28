use {
    super::ToServerInputMessage,
    crate::{
        Play,
        game_time::PingRequest,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum UdpToServerMessage<Game: Play> {
    //TODO: see if these can be borrowed
    PingRequest(PingRequest),
    Input(ToServerInputMessage<Game>),
}

impl<Game: Play> UdpToServerMessage<Game> {
    pub fn get_player_index(&self) -> usize {
        return match self {
            UdpToServerMessage::Input(input_message) => {
                input_message.get_input_coordinates().get_player_index()
            }
            UdpToServerMessage::PingRequest(ping_request) => ping_request.get_player_index(),
        };
    }
}
