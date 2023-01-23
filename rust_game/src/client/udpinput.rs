use std::net::{UdpSocket, SocketAddrV4, SocketAddr};
use crate::gametime::{TimeReceived, TimeValue, TimeDuration, GameTimer, GameTimerEvent};
use crate::messaging::{ToClientMessageUDP, MAX_UDP_DATAGRAM_SIZE, MessageFragment, FragmentAssembler};
use crate::threading::{ChannelDrivenThreadSender, eventhandling, listener};
use crate::interface::GameTrait;
use std::io;
use std::ops::ControlFlow::{Break, Continue};
use log::{debug, error, info, warn};
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::gamemanager::Manager;
use crate::threading::eventhandling::EventHandlerTrait;
use crate::threading::listener::{ListenedValueHolder, ListenerEventResult, ListenerTrait, ListenResult};
use crate::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};

pub struct UdpInput<Game: GameTrait> {
    server_socket_addr: SocketAddr,
    socket: UdpSocket,
    fragment_assembler: FragmentAssembler,
    game_timer_sender: eventhandling::Sender<GameTimer<ClientGameTimerObserver<Game>>>,
    manager_sender: ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
    time_of_last_server_input_receive: TimeValue,
}

impl<Game: GameTrait> UdpInput<Game> {

    pub fn new(
        server_socket_addr_v4: SocketAddrV4,
        socket: &UdpSocket,
        game_timer_sender: eventhandling::Sender<GameTimer<ClientGameTimerObserver<Game>>>,
        manager_sender: ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>) -> io::Result<Self> {

        let server_socket_addr = SocketAddr::from(server_socket_addr_v4);

        return Ok(Self{
            server_socket_addr,
            socket: socket.try_clone()?,
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(5),
            game_timer_sender,
            manager_sender,

            //metrics
            time_of_last_state_receive: TimeValue::now(),
            time_of_last_input_receive: TimeValue::now(),
            time_of_last_server_input_receive: TimeValue::now(),
        });
    }
}

impl<Game: GameTrait> ListenerTrait for UdpInput<Game> {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = ToClientMessageUDP<Game>;

    fn listen(mut self) -> ListenResult<Self> {
        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

        let recv_result = self.socket.recv_from(&mut buf);
        if recv_result.is_err() {
            warn!("Error on socket recv: {:?}", recv_result);
            return Continue(DidNotListen(self));
        }

        let (number_of_bytes, source) = recv_result.unwrap();

        if !self.server_socket_addr.eq(&source) {
            warn!("Received from wrong source. Expected: {:?}, Actual: {:?}", self.server_socket_addr, source);
            return Continue(DidNotListen(self));
        }

        let filled_buf = &mut buf[..number_of_bytes];
        let fragment = MessageFragment::from_vec(filled_buf.to_vec());

        if let Some(message_buf) = self.fragment_assembler.add_fragment(fragment) {

            match rmp_serde::from_read_ref(&message_buf) {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    return Continue(Listened(self, message));
                }
                Err(error) => {
                    error!("Error: {:?}", error);
                }
            }
        }

        return Continue(DidNotListen(self));
    }

    fn on_channel_event(self, event: listener::ChannelEvent<Self>) -> ListenerEventResult<Self> {
        return match event {
            listener::ChannelEvent::ChannelEmptyAfterListen(listened_value_holder) => self.handle_received_message(listened_value_holder),
            listener::ChannelEvent::ReceivedEvent(received_event_holder) => match received_event_holder.move_event() {
                () => {
                    warn!("This listener doesn't have meaningful messages, but one was sent.");
                    Continue(self)
                }
            },
            listener::ChannelEvent::ChannelDisconnected => Break(self.on_stop())
        };
    }

    fn on_stop(self) -> Self::ThreadReturn { () }
}

impl<Game: GameTrait> UdpInput<Game> {

    fn handle_received_message(mut self, listened_value_holder: ListenedValueHolder<Self>) -> ListenerEventResult<Self> {

        let time_received = listened_value_holder.get_time_received();

        match listened_value_holder.move_value() {
            ToClientMessageUDP::TimeMessage(time_message) => {
                //info!("Time message: {:?}", time_message.get_step());
                self.game_timer_sender.send_event(GameTimerEvent::TimeMessageEvent(TimeReceived::new(time_received, time_message))).unwrap();
            }
            ToClientMessageUDP::InputMessage(input_message) => {
                //TODO: ignore input messages from this player
                //info!("Input message: {:?}", input_message.get_step());
                self.time_of_last_input_receive = time_received;
                self.manager_sender.on_input_message(input_message.clone());
            }
            ToClientMessageUDP::ServerInputMessage(server_input_message) => {
                //info!("Server Input message: {:?}", server_input_message.get_step());
                self.time_of_last_server_input_receive = time_received;
                self.manager_sender.on_server_input_message(server_input_message);
            }
            ToClientMessageUDP::StateMessage(state_message) => {
                //info!("State message: {:?}", state_message.get_sequence());

                let duration_since_last_state = time_received.duration_since(self.time_of_last_state_receive);
                if duration_since_last_state > TimeDuration::one_second() {

                    //TODO: this should probably be a warn
                    debug!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                            duration_since_last_state, time_received, self.time_of_last_state_receive);
                }

                self.time_of_last_state_receive = time_received;
                self.manager_sender.on_state_message(state_message);
            }
        };

        return Continue(self);
    }
}
