use log::{error, info, warn};
use std::net::TcpStream;
use crate::gametime::GameTimer;
use crate::threading::{ChannelDrivenThreadSender, ValueSender};
use crate::messaging::ToClientMessageTCP;
use std::io;
use std::ops::ControlFlow::*;
use crate::client::ClientCore;
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::UdpOutput;
use crate::gamemanager::{Manager, RenderReceiverMessage};
use crate::interface::GameTrait;
use crate::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, WaitOrTryForNextEvent};

pub struct TcpInput <Game: GameTrait> {
    player_index: Option<usize>,
    tcp_stream: TcpStream,
    game_timer_sender: ChannelDrivenThreadSender<GameTimer<ClientGameTimerObserver<Game>>>,
    manager_sender: ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>,
    client_core_sender: ChannelDrivenThreadSender<ClientCore<Game>>,
    udp_output_sender: ChannelDrivenThreadSender<UdpOutput<Game>>,
    render_data_sender: ValueSender<RenderReceiverMessage<Game>>,
    received_message_option: Option<ToClientMessageTCP<Game>>
}

impl<Game: GameTrait> TcpInput<Game> {

    pub fn new(
        game_timer_sender: ChannelDrivenThreadSender<GameTimer<ClientGameTimerObserver<Game>>>,
        manager_sender: ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>,
        client_core_sender: ChannelDrivenThreadSender<ClientCore<Game>>,
        udp_output_sender: ChannelDrivenThreadSender<UdpOutput<Game>>,
        render_data_sender: ValueSender<RenderReceiverMessage<Game>>,
        tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(Self {
            player_index: None,
            tcp_stream: tcp_stream.try_clone()?,
            game_timer_sender,
            manager_sender,
            client_core_sender,
            udp_output_sender,
            render_data_sender,
            received_message_option: None
        })
    }
}

impl<Game: GameTrait> EventHandlerTrait for TcpInput<Game> {
    type Event = ();
    type ThreadReturn = ();

    fn on_channel_event(mut self, event: ChannelEvent<Self>) -> ChannelEventResult<Self> {
        return match event {
            ChannelEvent::ReceivedEvent(_) => {
                warn!("This handler does not have any meaningful messages");
                Continue(WaitOrTryForNextEvent::TryForNextEvent(self))
            }
            ChannelEvent::ChannelEmpty => {
                self.handle_received_message();
                self.wait_for_message()
            }
            ChannelEvent::ChannelDisconnected => Break(self.on_stop())
        };
    }

    fn on_stop(self) -> Self::ThreadReturn {
        return ();
    }
}

impl<Game: GameTrait> TcpInput<Game> {

    fn handle_received_message(&mut self) {

        if let Some(message) = self.received_message_option.take() {
            match message {
                ToClientMessageTCP::InitialInformation(initial_information_message) => {

                    info!("InitialInformation Received.");

                    self.player_index = Some(initial_information_message.get_player_index());
                    self.game_timer_sender.on_initial_information(initial_information_message.clone());
                    self.manager_sender.on_initial_information(initial_information_message.clone());
                    self.client_core_sender.on_initial_information(initial_information_message.clone());
                    self.udp_output_sender.on_initial_information(initial_information_message.clone());
                    self.render_data_sender.send(RenderReceiverMessage::InitialInformation(initial_information_message)).unwrap();
                }
            }
        }
    }

    fn wait_for_message(mut self) -> ChannelEventResult<Self> {
        return match rmp_serde::from_read(&self.tcp_stream) {
            Ok(message) => {

                //Why does this crash the client?
                //info!("{:?}", message);

                self.received_message_option = Some(message);
                Continue(WaitOrTryForNextEvent::TryForNextEvent(self))
            }
            Err(error) => {
                error!("Error: {:?}", error);
                Break(self.on_stop())
            }
        }
    }
}