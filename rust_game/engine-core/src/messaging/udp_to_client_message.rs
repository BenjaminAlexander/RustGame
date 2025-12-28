use {
    crate::{
        Play,
        frame_manager::FrameIndexAndState,
        game_time::PingResponse,
        messaging::ToClientInputMessage,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum UdpToClientMessage<Game: Play> {
    //TODO: see if these can be borrowed
    PingResponse(PingResponse),
    InputMessage(ToClientInputMessage<Game>),
    StateMessage(FrameIndexAndState<Game>),
}
