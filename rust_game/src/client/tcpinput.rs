use log::{error, info};
use std::net::TcpStream;
use crate::gametime::{GameTimer, TimeValue};
use crate::threading::{ChannelThread, Receiver, ChannelDrivenThreadSender as Sender, ThreadAction, MessageChannelSender};
use crate::messaging::ToClientMessageTCP;
use rmp_serde::decode::Error;
use std::io;
use std::sync::mpsc::TryRecvError;
use crate::client::ClientCore;
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::UdpOutput;
use crate::gamemanager::{Manager, RenderReceiverMessage};
use crate::interface::GameTrait;

pub struct TcpInput <Game: GameTrait> {
    player_index: Option<usize>,
    tcp_stream: TcpStream,
    game_timer_sender: Sender<GameTimer<ClientGameTimerObserver<Game>>>,
    manager_sender: Sender<Manager<ClientManagerObserver<Game>>>,
    client_core_sender: Sender<ClientCore<Game>>,
    udp_output_sender: Sender<UdpOutput<Game>>,
    render_data_sender: MessageChannelSender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> TcpInput<Game> {

    pub fn new(
        game_timer_sender: Sender<GameTimer<ClientGameTimerObserver<Game>>>,
        manager_sender: Sender<Manager<ClientManagerObserver<Game>>>,
        client_core_sender: Sender<ClientCore<Game>>,
        udp_output_sender: Sender<UdpOutput<Game>>,
        render_data_sender: MessageChannelSender<RenderReceiverMessage<Game>>,
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

impl<Game: GameTrait> ChannelThread<(), ThreadAction> for TcpInput<Game> {

    fn run(mut self, receiver: Receiver<Self, ThreadAction>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToClientMessageTCP::<Game>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    let _time_received = TimeValue::now();

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

                    info!("InitialInformation Received.");

                    match message {
                        ToClientMessageTCP::InitialInformation(initial_information_message) => {
                            self.player_index = Some(initial_information_message.get_player_index());
                            self.game_timer_sender.on_initial_information(initial_information_message.clone());
                            self.manager_sender.on_initial_information(initial_information_message.clone());
                            self.client_core_sender.on_initial_information(initial_information_message.clone());
                            self.udp_output_sender.on_initial_information(initial_information_message.clone());
                            self.render_data_sender.send(RenderReceiverMessage::InitialInformation(initial_information_message.clone())).unwrap();
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