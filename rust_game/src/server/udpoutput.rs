use log::{trace, info, warn, error};
use crate::interface::GameTrait;
use std::net::UdpSocket;
use crate::gametime::{TimeDuration, TimeMessage, TimeValue};
use crate::messaging::{InputMessage, StateMessage, ToClientMessageUDP, Fragmenter, MAX_UDP_DATAGRAM_SIZE, ServerInputMessage};
use std::io;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::threading::{ChannelThread, Receiver, Sender, Consumer};
use std::time::Duration;
use std::sync::mpsc::RecvTimeoutError;
use std::marker::PhantomData;
use crate::util::RollingAverage;

pub struct UdpOutput<Game: GameTrait> {
    player_index: usize,
    socket: UdpSocket,
    remote_peer: Option<RemoteUdpPeer>,
    fragmenter: Fragmenter,
    last_time_message: Option<TimeMessage>,
    last_state_sequence: Option<usize>,
    phantom: PhantomData<Game>,

    //metrics
    time_in_queue_rolling_average: RollingAverage<u64>,
    time_of_last_state_send: TimeValue,
    time_of_last_input_send: TimeValue,
    time_of_last_server_input_send: TimeValue,
}

impl<Game: GameTrait> UdpOutput<Game> {

    pub fn new(player_index: usize, socket: &UdpSocket) -> io::Result<Self> {

        Ok(UdpOutput{
            player_index,
            remote_peer: None,
            socket: socket.try_clone()?,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            last_time_message: None,
            last_state_sequence: None,
            phantom: PhantomData,

            //metrics
            time_in_queue_rolling_average: RollingAverage::new(100),
            time_of_last_state_send: TimeValue::now(),
            time_of_last_input_send: TimeValue::now(),
            time_of_last_server_input_send: TimeValue::now(),
        })
    }

    fn log_time_in_queue(&mut self, time_in_queue: TimeValue) {
        let now = TimeValue::now();
        let duration_in_queue = now.duration_since(time_in_queue);

        self.time_in_queue_rolling_average.add_value(duration_in_queue.get_millis() as u64);
        let average = self.time_in_queue_rolling_average.get_average();

        if average > 500 {
            warn!("High average duration in queue: {:?} in milliseconds", average);
        }
    }

    fn send_message(&mut self, message: ToClientMessageUDP<Game>) {

        if let Some(remote_peer) = &self.remote_peer {
            let buf = rmp_serde::to_vec(&message).unwrap();
            let fragments = self.fragmenter.make_fragments(buf);

            for fragment in fragments {

                if fragment.get_whole_buf().len() > MAX_UDP_DATAGRAM_SIZE {
                    error!("Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}", fragment.get_whole_buf().len());
                }

                self.socket.send_to(fragment.get_whole_buf(), remote_peer.get_socket_addr()).unwrap();
            }
        }
    }
}

impl<Game: GameTrait> ChannelThread<()> for UdpOutput<Game> {

    fn run(mut self, receiver: Receiver<Self>) -> () {

        loop {
            trace!("Waiting.");

            match receiver.recv_timeout(&mut self, Duration::new(1, 0)) {
                Err(error) => {
                    match error {
                        RecvTimeoutError::Timeout => { }
                        RecvTimeoutError::Disconnected => {
                            info!("Channel closed.");
                            return ();
                        }
                    }
                }
                _ => {}
            }

            let now = TimeValue::now();

            // let duration_since_last_input = now.duration_since(self.time_of_last_input_send);
            // if duration_since_last_input > TimeDuration::one_second() {
            //     warn!("It has been {:?} since last input message was sent. Now: {:?}, Last: {:?}, Queue length: {:?}",
            //           duration_since_last_input, now, self.time_of_last_input_send, self.input_queue.len());
            // }

            let duration_since_last_state = now.duration_since(self.time_of_last_state_send);
            if duration_since_last_state > TimeDuration::one_second() {
                warn!("It has been {:?} since last state message was sent. Now: {:?}, Last: {:?}",
                      duration_since_last_state, now, self.time_of_last_state_send);
            }
        }
    }
}

impl<Game: GameTrait> Sender<UdpOutput<Game>> {

    pub fn on_remote_peer(&self, remote_peer: RemoteUdpPeer) {
        self.send(|udp_output|{

            if udp_output.player_index == remote_peer.get_player_index() {
                info!("Setting remote peer: {:?}", remote_peer);
                udp_output.remote_peer = Some(remote_peer);
            }
        }).unwrap();
    }

    pub fn on_completed_step(&self, state_message: StateMessage<Game>) {

        let time_in_queue = TimeValue::now();

        //info!("state_message: {:?}", state_message.get_sequence());

        self.send(move |udp_output|{

            if udp_output.last_state_sequence.is_none() ||
                udp_output.last_state_sequence.as_ref().unwrap() <= &state_message.get_sequence() {

                udp_output.last_state_sequence = Some(state_message.get_sequence());
                udp_output.time_of_last_state_send = TimeValue::now();

                let message = ToClientMessageUDP::<Game>::StateMessage(state_message);
                udp_output.send_message(message);

                //info!("state_message");
                udp_output.log_time_in_queue(time_in_queue);

            }
        }).unwrap();
    }

    pub fn on_time_message(&self, time_message: TimeMessage) {

        let time_in_queue = TimeValue::now();

        self.send(move |udp_output|{

            let mut send_it = false;

            if let Some(last_time_message) = &udp_output.last_time_message {
                if time_message.get_scheduled_time().is_after(&last_time_message.get_scheduled_time().add(Game::TIME_SYNC_MESSAGE_PERIOD)) {
                    send_it = true;
                }
            } else {
                send_it = true;
            }

            if send_it {

                udp_output.last_time_message = Some(time_message.clone());

                //TODO: timestamp when the time message is set, then use that info in client side time calc
                let message = ToClientMessageUDP::<Game>::TimeMessage(time_message);
                udp_output.send_message(message);

                //info!("time_message");
                udp_output.log_time_in_queue(time_in_queue);
            }

        }).unwrap();
    }

    pub fn on_input_message(&self, input_message: InputMessage<Game>) {

        let time_in_queue = TimeValue::now();

        self.send(move |udp_output|{

            if udp_output.player_index != input_message.get_player_index() &&
                (udp_output.last_state_sequence.is_none() ||
                    udp_output.last_state_sequence.as_ref().unwrap() <= &input_message.get_step()) {

                udp_output.time_of_last_input_send = TimeValue::now();

                let message = ToClientMessageUDP::<Game>::InputMessage(input_message);
                udp_output.send_message(message);

                //info!("input_message");
                udp_output.log_time_in_queue(time_in_queue);
            } else {
                //info!("InputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
            }
        }).unwrap();
    }
}

impl<Game: GameTrait> Consumer<ServerInputMessage<Game>> for Sender<UdpOutput<Game>> {

    fn accept(&self, server_input_message: ServerInputMessage<Game>) {

        let time_in_queue = TimeValue::now();

        self.send(move |udp_output|{

            if udp_output.last_state_sequence.is_none() ||
               udp_output.last_state_sequence.as_ref().unwrap() <= &server_input_message.get_step() {

                udp_output.time_of_last_server_input_send = TimeValue::now();

                let message = ToClientMessageUDP::<Game>::ServerInputMessage(server_input_message);
                udp_output.send_message(message);

                //info!("server_input_message");
                udp_output.log_time_in_queue(time_in_queue);
            } else {
                //info!("ServerInputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
            }
        }).unwrap();
    }
}