use log::{trace, info};
use std::net::TcpStream;
use crate::threading::{Sender, ChannelThread, Receiver};
use std::io;
use crate::messaging::{ToClientMessageTCP, InitialInformation};
use std::io::Write;
use crate::interface::Game;
use std::marker::PhantomData;
use crate::server::ServerConfig;

pub struct TcpOutput<GameType: Game> {
    player_index: usize,
    tcp_stream: TcpStream,
    phantom: PhantomData<GameType>
}

impl<GameType: Game> TcpOutput<GameType> {

    pub fn new(player_index: usize,
               tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(TcpOutput{
            player_index,
            tcp_stream: tcp_stream.try_clone()?,
            phantom: PhantomData
        })
    }
}

impl<GameType: Game> ChannelThread<()> for TcpOutput<GameType> {

    fn run(mut self, receiver: Receiver<Self>) -> () {

        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    info!("Channel closed.");
                    return ();
                }
                _ => {}
            }

            receiver.try_iter(&mut self);
        }
    }
}

impl<GameType: Game> Sender<TcpOutput<GameType>> {

    pub fn send_initial_information(&self, server_config: ServerConfig, player_count: usize, initial_state: GameType::StateType) {
        self.send(move |tcp_output|{

            let initial_information = InitialInformation::<GameType>::new(
                server_config,
                player_count,
                tcp_output.player_index,
                initial_state
            );

            let message = ToClientMessageTCP::<GameType>::InitialInformation(initial_information);
            rmp_serde::encode::write(&mut tcp_output.tcp_stream, &message).unwrap();
            tcp_output.tcp_stream.flush().unwrap();

        }).unwrap();
    }
}