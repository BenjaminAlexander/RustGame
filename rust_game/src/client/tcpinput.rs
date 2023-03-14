use log::{error, info, warn};
use std::net::TcpStream;
use commons::threading::{channel, eventhandling};
use crate::messaging::ToClientMessageTCP;
use std::io;
use std::ops::ControlFlow::*;
use crate::client::clientcore::ClientCoreEvent;
use crate::client::ClientCoreEvent::OnInitialInformation;
use crate::client::udpoutput::UdpOutputEvent;
use crate::gamemanager::{ManagerEvent, RenderReceiverMessage};
use crate::interface::GameTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::listener::{ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use commons::threading::listener::ListenedOrDidNotListen::Listened;

pub struct TcpInput <Game: GameTrait> {
    player_index: Option<usize>,
    tcp_stream: TcpStream,
    manager_sender: eventhandling::Sender<ManagerEvent<Game>>,
    client_core_sender: eventhandling::Sender<ClientCoreEvent<Game>>,
    render_data_sender: channel::Sender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> TcpInput<Game> {

    pub fn new(
        manager_sender: eventhandling::Sender<ManagerEvent<Game>>,
        client_core_sender: eventhandling::Sender<ClientCoreEvent<Game>>,
        render_data_sender: channel::Sender<RenderReceiverMessage<Game>>,
        tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(Self {
            player_index: None,
            tcp_stream: tcp_stream.try_clone()?,
            manager_sender,
            client_core_sender,
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
            ChannelEvent::ChannelEmptyAfterListen(_, value) => {
                self.handle_received_message(value);
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
                self.manager_sender.send_event(ManagerEvent::InitialInformationEvent(initial_information_message.clone())).unwrap();
                self.client_core_sender.send_event(OnInitialInformation(initial_information_message.clone())).unwrap();
                self.render_data_sender.send(RenderReceiverMessage::InitialInformation(initial_information_message)).unwrap();
            }
        }
    }
}