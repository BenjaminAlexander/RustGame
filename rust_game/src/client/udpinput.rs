use std::net::{UdpSocket, SocketAddrV4, SocketAddr};
use crate::gametime::{TimeReceived, TimeValue, TimeDuration, GameTimer};
use crate::messaging::{ToClientMessageUDP, MAX_UDP_DATAGRAM_SIZE, MessageFragment, FragmentAssembler};
use crate::threading::{ChannelDrivenThreadSender as Sender, EventHandlerTrait, WaitOrTry, ChannelEvent, EventHandlerResult};
use crate::interface::GameTrait;
use std::io;
use std::ops::ControlFlow::{Break, Continue};
use log::{debug, error, warn};
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::gamemanager::Manager;

pub struct UdpInput<Game: GameTrait> {
    server_socket_addr: SocketAddr,
    socket: UdpSocket,
    fragment_assembler: FragmentAssembler,
    game_timer_sender: Sender<GameTimer<ClientGameTimerObserver<Game>>>,
    manager_sender: Sender<Manager<ClientManagerObserver<Game>>>,
    received_message_option: Option<(ToClientMessageUDP<Game>, TimeValue)>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
    time_of_last_server_input_receive: TimeValue,
}

impl<Game: GameTrait> UdpInput<Game> {

    pub fn new(
        server_socket_addr_v4: SocketAddrV4,
        socket: &UdpSocket,
        game_timer_sender: Sender<GameTimer<ClientGameTimerObserver<Game>>>,
        manager_sender: Sender<Manager<ClientManagerObserver<Game>>>) -> io::Result<Self> {

        let server_socket_addr = SocketAddr::from(server_socket_addr_v4);

        return Ok(Self{
            server_socket_addr,
            socket: socket.try_clone()?,
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(5),
            game_timer_sender,
            manager_sender,
            received_message_option: None,

            //metrics
            time_of_last_state_receive: TimeValue::now(),
            time_of_last_input_receive: TimeValue::now(),
            time_of_last_server_input_receive: TimeValue::now(),
        });
    }
}

impl<Game: GameTrait> EventHandlerTrait for UdpInput<Game> {
    type Event = ();
    type ThreadReturnType = ();

    fn on_event(mut self, event: ChannelEvent<Self>) -> EventHandlerResult<Self> {
        return match event {
            ChannelEvent::ReceivedEvent(_) => {
                warn!("This handler does not have any meaningful messages");
                Continue(WaitOrTry::TryForNextEvent(self))
            }
            ChannelEvent::ChannelEmpty => {
                self.handle_received_message();
                self.wait_for_message()
            }
            ChannelEvent::ChannelDisconnected => Break(self.on_stop())
        };
    }

    fn on_stop(self) -> Self::ThreadReturnType {
        return ();
    }
}

impl<Game: GameTrait> UdpInput<Game> {

    fn handle_received_message(&mut self) {
        if let Some((message, time_received)) = self.received_message_option.take() {

            match message {
                ToClientMessageUDP::TimeMessage(time_message) => {
                    //info!("Time message: {:?}", time_message.get_step());
                    self.game_timer_sender.on_time_message(TimeReceived::new(time_received, time_message));
                }
                ToClientMessageUDP::InputMessage(input_message) => {
                    //TODO: ignore input messages from this player
                    //info!("Input message: {:?}", input_message.get_step());
                    self.time_of_last_input_receive = TimeValue::now();
                    self.manager_sender.on_input_message(input_message.clone());
                }
                ToClientMessageUDP::ServerInputMessage(server_input_message) => {
                    //info!("Server Input message: {:?}", server_input_message.get_step());
                    self.time_of_last_server_input_receive = TimeValue::now();
                    self.manager_sender.on_server_input_message(server_input_message);
                }
                ToClientMessageUDP::StateMessage(state_message) => {
                    //info!("State message: {:?}", state_message.get_sequence());

                    let now = TimeValue::now();
                    let duration_since_last_state = now.duration_since(self.time_of_last_state_receive);
                    if duration_since_last_state > TimeDuration::one_second() {

                        //TODO: this should probably be a warn
                        debug!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                                duration_since_last_state, now, self.time_of_last_state_receive);
                    }

                    self.time_of_last_state_receive = now;
                    self.manager_sender.on_state_message(state_message);
                }
            }
        }
    }

    fn wait_for_message(mut self) -> EventHandlerResult<Self> {

        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

        let recv_result = self.socket.recv_from(&mut buf);
        if recv_result.is_err() {
            warn!("Error on socket recv: {:?}", recv_result);
            return Continue(WaitOrTry::TryForNextEvent(self));
        }

        let (number_of_bytes, source) = recv_result.unwrap();

        if !self.server_socket_addr.eq(&source) {
            warn!("Received from wrong source. Expected: {:?}, Actual: {:?}", self.server_socket_addr, source);
            return Continue(WaitOrTry::TryForNextEvent(self));
        }

        let filled_buf = &mut buf[..number_of_bytes];
        let fragment = MessageFragment::from_vec(filled_buf.to_vec());

        if let Some(message_buf) = self.fragment_assembler.add_fragment(fragment) {

            match rmp_serde::from_read_ref(&message_buf) {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    self.received_message_option = Some((message, TimeValue::now()))
                }
                Err(error) => {
                    error!("Error: {:?}", error);

                }
            }
        }

        return Continue(WaitOrTry::TryForNextEvent(self));
    }
}
