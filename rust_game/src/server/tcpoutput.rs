use log::{debug, info};
use std::net::TcpStream;
use crate::threading::{ChannelDrivenThreadSender as Sender, ChannelThread, Receiver, ThreadAction};
use std::io;
use crate::messaging::{ToClientMessageTCP, InitialInformation};
use std::io::Write;
use crate::interface::GameTrait;
use std::marker::PhantomData;
use crate::server::ServerConfig;

pub struct TcpOutput<Game: GameTrait> {
    player_index: usize,
    tcp_stream: TcpStream,
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> TcpOutput<Game> {

    pub fn new(player_index: usize,
               tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(TcpOutput{
            player_index,
            tcp_stream: tcp_stream.try_clone()?,
            phantom: PhantomData
        })
    }
}

impl<Game: GameTrait> ChannelThread<(), ThreadAction> for TcpOutput<Game> {

    fn run(mut self, receiver: Receiver<Self, ThreadAction>) -> () {

        loop {

            match receiver.recv(&mut self) {
                Err(error) => {

                    info!("Channel closed: {:?}", error);
                    return ();
                }
                _ => {}
            }
        }
    }
}

impl<Game: GameTrait> Sender<TcpOutput<Game>> {

    pub fn send_initial_information(&self, server_config: ServerConfig, player_count: usize, initial_state: Game::State) {
        self.send(move |tcp_output|{

            let initial_information = InitialInformation::<Game>::new(
                server_config,
                player_count,
                tcp_output.player_index,
                initial_state
            );

            let message = ToClientMessageTCP::<Game>::InitialInformation(initial_information);
            rmp_serde::encode::write(&mut tcp_output.tcp_stream, &message).unwrap();
            tcp_output.tcp_stream.flush().unwrap();

            debug!("Sent InitialInformation");

            return ThreadAction::Continue;
        }).unwrap();
    }
}