use log::{error, info};
use std::net::TcpStream;
use crate::gametime::{GameTimer, TimeValue};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::messaging::{ToClientMessageTCP, InitialInformation};
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use std::io;
use crate::client::ClientCore;
use crate::client::udpoutput::UdpOutput;
use crate::gamemanager::{Data, Manager};
use crate::interface::GameTrait;

pub struct TcpInput <Game: GameTrait> {
    player_index: Option<usize>,
    tcp_stream: TcpStream,
    game_timer_sender: Sender<GameTimer<Sender<ClientCore<Game>>>>,
    manager_sender: Sender<Manager<Sender<ClientCore<Game>>>>,
    client_core_sender: Sender<ClientCore<Game>>,
    udp_output_sender: Sender<UdpOutput<Game>>,
    render_data_sender: Sender<Data<Game>>
}

impl<Game: GameTrait> TcpInput<Game> {

    pub fn new(
        game_timer_sender: Sender<GameTimer<Sender<ClientCore<Game>>>>,
        manager_sender: Sender<Manager<Sender<ClientCore<Game>>>>,
        client_core_sender: Sender<ClientCore<Game>>,
        udp_output_sender: Sender<UdpOutput<Game>>,
        render_data_sender: Sender<Data<Game>>,
        tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(Self {
            player_index: None,
            tcp_stream: tcp_stream.try_clone()?,
            game_timer_sender,
            manager_sender,
            client_core_sender,
            udp_output_sender,
            render_data_sender
        })
    }
}

impl<Game: GameTrait> ChannelThread<()> for TcpInput<Game> {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToClientMessageTCP::<Game>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    let _time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToClientMessageTCP::InitialInformation(initial_information_message) => {
                            self.player_index = Some(initial_information_message.get_player_index());
                            self.game_timer_sender.on_initial_information(initial_information_message.clone());
                            self.manager_sender.on_initial_information(initial_information_message.clone());
                            self.client_core_sender.on_initial_information(initial_information_message.clone());
                            self.udp_output_sender.on_initial_information(initial_information_message.clone());
                            self.render_data_sender.on_initial_information(initial_information_message.clone());
                        }
                    }
                }
                Err(error) => {
                    error!("Error: {:?}", error);
                    return;
                }
            }
        }
    }
}