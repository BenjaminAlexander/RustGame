use std::net::TcpStream;

use log::{error, info};
use rmp_serde::decode::Error;

use crate::messaging::{ToServerMessageTCP};
use crate::threading::{ChannelThread, Receiver, ThreadAction};
use crate::gametime::TimeValue;
use std::io;
use std::sync::mpsc::TryRecvError;

pub struct TcpInput {
    tcp_stream: TcpStream
}

impl TcpInput {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {tcp_stream: tcp_stream.try_clone()?})
    }
}

impl ChannelThread<(), ThreadAction> for TcpInput {

    //TODO: check player ID on message
    fn run(mut self, receiver: Receiver<Self, ThreadAction>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToServerMessageTCP, Error> = rmp_serde::from_read(&self.tcp_stream);

            loop {
                match receiver.try_recv(&mut self) {
                    Ok(ThreadAction::Continue) => {}
                    Err(TryRecvError::Empty) => break,
                    Ok(ThreadAction::Stop) => {
                        info!("Thread commanded to stop.");
                        return;
                    }
                    Err(TryRecvError::Disconnected) => {
                        info!("Thread stopping due to disconnect.");
                        return;
                    }
                }
            }

            match result {
                Ok(message) => {
                    let time_received = TimeValue::now();
                    match message {

                    };
                }
                Err(error) => {
                    error!("rmp_serde Error: {:?}", error);
                    //return;
                }
            }

            loop {
                match receiver.try_recv(&mut self) {
                    Ok(ThreadAction::Continue) => {}
                    Err(TryRecvError::Empty) => break,
                    Ok(ThreadAction::Stop) => {
                        info!("Thread commanded to stop.");
                        return;
                    }
                    Err(TryRecvError::Disconnected) => {
                        info!("Thread stopping due to disconnect.");
                        return;
                    }
                }
            }
        }
    }
}