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

pub use {
    self::{
        aggregate_input::AggregateInput,
        client::Client,
        current_frame::CurrentFrame,
        frame_manager::{
            ClientProvidedInput,
            FrameInput,
            FrameState,
            PresentInput,
        },
        game_time::{
            FrameDuration,
            FrameIndex,
            StartTime,
        },
        play::Play,
        smooth::{
            Smooth,
            SmoothArguments,
        },
        smooth_receiver::SmoothReceiver,
        state_channel::{
            MostRecentState,
            StateReceiver,
            StateReceiverError,
        },
    },
    interface::{
        InitialInformation,
        Server,
    },
};
