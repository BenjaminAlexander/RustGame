#![doc = include_str!("../README.md")]

#![deny(rustdoc::broken_intra_doc_links)]
mod aggregate_input;
mod client;
mod current_frame;
mod frame_manager;
mod game_time;
mod interface;
mod messaging;
mod play;
mod server;
mod smooth;
mod smooth_receiver;
mod state_channel;

pub use self::aggregate_input::AggregateInput;
pub use self::current_frame::CurrentFrame;
pub use self::frame_manager::{
    ClientProvidedInput,
    FrameInput,
    FrameState,
    PresentInput,
};
pub use self::game_time::{
    FrameDuration,
    FrameIndex,
    StartTime
};
pub use self::play::Play;
pub use self::smooth::Smooth;
pub use self::smooth::SmoothArguments;
pub use self::smooth_receiver::SmoothReceiver;
pub use self::state_channel::{
    MostRecentState,
    StateReceiver,
    StateReceiverError,
};
pub use interface::Client;
pub use interface::InitialInformation;
pub use interface::Server;
