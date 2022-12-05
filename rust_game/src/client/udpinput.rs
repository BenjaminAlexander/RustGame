use std::net::{UdpSocket, SocketAddrV4, SocketAddr};
use crate::gametime::{TimeReceived, TimeValue, TimeDuration, GameTimer};
use crate::messaging::{ToClientMessageUDP, MAX_UDP_DATAGRAM_SIZE, MessageFragment, FragmentAssembler};
use crate::threading::{Sender, Receiver, ChannelThread};
use crate::interface::GameTrait;
use rmp_serde::decode::Error;
use std::io;
use log::{error, info, warn};
use std::time::Duration;
use crate::client::ClientCore;
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::gamemanager::Manager;

pub struct UdpInput<Game: GameTrait> {
    server_socket_addr: SocketAddr,
    socket: UdpSocket,
    fragment_assembler: FragmentAssembler,
    game_timer_sender: Sender<GameTimer<ClientGameTimerObserver<Game>>>,
    manager_sender: Sender<Manager<Sender<ClientCore<Game>>>>,

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
        manager_sender: Sender<Manager<Sender<ClientCore<Game>>>>) -> io::Result<Self> {

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

impl<Game: GameTrait> ChannelThread<()> for UdpInput<Game> {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        self.socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();

        loop {

            let now = TimeValue::now();
            let duration_since_last_state = now.duration_since(self.time_of_last_state_receive);
            if duration_since_last_state > TimeDuration::one_second() {
                warn!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                      duration_since_last_state, now, self.time_of_last_state_receive);
            }

            let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

            let recv_result = self.socket.recv_from(&mut buf);
            if recv_result.is_err() {
                warn!("Error on socket recv: {:?}", recv_result);
                continue;
            }

            let (number_of_bytes, source) = recv_result.unwrap();

            if !self.server_socket_addr.eq(&source) {
                warn!("Received from wrong source. Expected: {:?}, Actual: {:?}", self.server_socket_addr, source);
                continue;
            }

            let filled_buf = &mut buf[..number_of_bytes];
            let fragment = MessageFragment::from_vec(filled_buf.to_vec());

            if let Some(message_buf) = self.fragment_assembler.add_fragment(fragment) {

                let result: Result<ToClientMessageUDP<Game>, Error> = rmp_serde::from_read_ref(&message_buf);

                match result {
                    Ok(message) => {

                        //Why does this crash the client?
                        //info!("{:?}", message);

                        let time_received = TimeValue::now();

                        receiver.try_iter(&mut self);

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
                                self.time_of_last_state_receive = TimeValue::now();
                                self.manager_sender.on_state_message(state_message);

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
}