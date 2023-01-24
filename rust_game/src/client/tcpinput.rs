use log::{error, info, warn};
use std::net::TcpStream;
use crate::gametime::{GameTimer, GameTimerEvent};
use crate::threading::{ChannelDrivenThreadSender, eventhandling};
use crate::messaging::ToClientMessageTCP;
use std::io;
use std::ops::ControlFlow::*;
use crate::client::ClientCore;
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::gamemanager::{Manager, RenderReceiverMessage};
use crate::interface::GameTrait;
use crate::threading::channel::{ReceiveMetaData, Sender};
use crate::threading::listener::{ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use crate::threading::listener::ListenedOrDidNotListen::Listened;

pub struct TcpInput <Game: GameTrait> {
    player_index: Option<usize>,
    tcp_stream: TcpStream,
    game_timer_sender: eventhandling::Sender<GameTimer<ClientGameTimerObserver<Game>>>,
    manager_sender: ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>,
    client_core_sender: ChannelDrivenThreadSender<ClientCore<Game>>,
    udp_output_sender: eventhandling::Sender<UdpOutput<Game>>,
    render_data_sender: Sender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> TcpInput<Game> {

    pub fn new(
        game_timer_sender: eventhandling::Sender<GameTimer<ClientGameTimerObserver<Game>>>,
        manager_sender: ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>,
        client_core_sender: ChannelDrivenThreadSender<ClientCore<Game>>,
        udp_output_sender: eventhandling::Sender<UdpOutput<Game>>,
        render_data_sender: Sender<RenderReceiverMessage<Game>>,
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

impl<Game: GameTrait> ListenerTrait for TcpInput<Game> {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = ToClientMessageTCP<Game>;

    fn listen(self) -> ListenResult<Self> {
        return match rmp_serde::from_read(&self.tcp_stream) {
            Ok(message) => Continue(Listened(self, message)),
            Err(error) => {
                error!("Error: {:?}", error);
                Break(())
            }
        }
    }

    fn on_channel_event(mut self, event: ChannelEvent<Self>) -> ListenerEventResult<Self> {
        match event {
            ChannelEvent::ChannelEmptyAfterListen(listened_value_holder) => {
                self.handle_received_message(listened_value_holder.move_value());
                Continue(self)
            }
            ChannelEvent::ReceivedEvent(_, ()) => {
                warn!("This handler does not have any meaningful messages");
                Continue(self)
            }
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}

impl<Game: GameTrait> TcpInput<Game> {

    fn handle_received_message(&mut self, message: ToClientMessageTCP<Game>) {

        match message {
            ToClientMessageTCP::InitialInformation(initial_information_message) => {
                info!("InitialInformation Received.  Player Index: {:?}", initial_information_message.get_player_index());

                self.player_index = Some(initial_information_message.get_player_index());
                self.game_timer_sender.send_event(GameTimerEvent::InitialInformationEvent(initial_information_message.clone())).unwrap();
                self.manager_sender.on_initial_information(initial_information_message.clone());
                self.client_core_sender.on_initial_information(initial_information_message.clone());
                self.udp_output_sender.send_event(UdpOutputEvent::InitialInformationEvent(initial_information_message.clone()));
                self.render_data_sender.send(RenderReceiverMessage::InitialInformation(initial_information_message)).unwrap();
            }
        }
    }
}