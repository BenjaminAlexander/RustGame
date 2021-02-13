use log::{trace, info, warn};
use crate::interface::{Input, State};
use std::net::{UdpSocket, SocketAddr};
use crate::gametime::{TimeDuration, TimeMessage, TimeValue};
use crate::messaging::{InputMessage, StateMessage, ToClientMessageUDP};
use std::io;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::threading::{ChannelThread, Receiver, Sender, Consumer};
use crate::server::tcpoutput::TcpOutput;
use std::time::Duration;
use std::sync::mpsc::RecvTimeoutError;
use std::marker::PhantomData;
use crate::util::RollingAverage;

pub struct UdpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    player_index: usize,
    socket: UdpSocket,
    remote_peer: Option<RemoteUdpPeer>,
    time_message_period: TimeDuration,
    last_time_message: Option<TimeMessage>,
    last_state_sequence: Option<usize>,
    state_phantom: PhantomData<StateType>,
    input_phantom: PhantomData<InputType>,

    //metrics
    time_in_queue_rolling_average: RollingAverage<u64>,
    time_of_last_state_send: TimeValue,
    time_of_last_input_send: TimeValue,
}

impl<StateType, InputType> UdpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn new(time_message_period: TimeDuration,
               player_index: usize,
               socket: &UdpSocket) -> io::Result<Self> {

        Ok(UdpOutput{
            player_index,
            remote_peer: None,
            socket: socket.try_clone()?,
            time_message_period,
            last_time_message: None,
            last_state_sequence: None,
            state_phantom: PhantomData,
            input_phantom: PhantomData,

            //metrics
            time_in_queue_rolling_average: RollingAverage::new(100),
            time_of_last_state_send: TimeValue::now(),
            time_of_last_input_send: TimeValue::now(),
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
}

impl<StateType, InputType> ChannelThread<()> for UdpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

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

impl<StateType, InputType> Consumer<TimeMessage> for Sender<UdpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, time_message: TimeMessage) {

        let time_in_queue = TimeValue::now();

        self.send(move |udp_output|{

            let mut send_it = false;

            if let Some(last_time_message) = &udp_output.last_time_message {
                if time_message.get_scheduled_time().is_after(&last_time_message.get_scheduled_time().add(udp_output.time_message_period)) {
                    send_it = true;
                }
            } else {
                send_it = true;
            }

            if send_it {

                udp_output.last_time_message = Some(time_message.clone());

                //TODO: timestamp when the time message is set, then use that info in client side time calc
                let message = ToClientMessageUDP::<StateType, InputType>::TimeMessage(time_message);

                let buf = rmp_serde::to_vec(&message).unwrap();
                if let Some(remote_peer) = &udp_output.remote_peer {
                    udp_output.socket.send_to(&buf, remote_peer.get_socket_addr()).unwrap();
                }

                //info!("time_message");
                udp_output.log_time_in_queue(time_in_queue);
            }

        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<UdpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, input_message: InputMessage<InputType>) {

        let time_in_queue = TimeValue::now();

        self.send(move |udp_output|{

            if udp_output.player_index != input_message.get_player_index() &&
                (udp_output.last_state_sequence.is_none() ||
                    udp_output.last_state_sequence.as_ref().unwrap() <= &input_message.get_step()) {

                udp_output.time_of_last_input_send = TimeValue::now();

                let message = ToClientMessageUDP::<StateType, InputType>::InputMessage(input_message);
                let buf = rmp_serde::to_vec(&message).unwrap();
                if let Some(remote_peer) = &udp_output.remote_peer {
                    udp_output.socket.send_to(&buf, remote_peer.get_socket_addr()).unwrap();
                }

                //info!("input_message");
                udp_output.log_time_in_queue(time_in_queue);
            } else {
                //info!("InputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<StateMessage<StateType>> for Sender<UdpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, state_message: StateMessage<StateType>) {

        let time_in_queue = TimeValue::now();

        self.send(move |udp_output|{

            if udp_output.last_state_sequence.is_none() ||
                udp_output.last_state_sequence.as_ref().unwrap() <= &state_message.get_sequence() {

                udp_output.last_state_sequence = Some(state_message.get_sequence());
                udp_output.time_of_last_state_send = TimeValue::now();

                let message = ToClientMessageUDP::<StateType, InputType>::StateMessage(state_message);
                let buf = rmp_serde::to_vec(&message).unwrap();
                if let Some(remote_peer) = &udp_output.remote_peer {
                    udp_output.socket.send_to(&buf, remote_peer.get_socket_addr()).unwrap();
                }
                //info!("state_message");
                udp_output.log_time_in_queue(time_in_queue);

            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<RemoteUdpPeer> for Sender<UdpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, remote_peer: RemoteUdpPeer) {
        self.send(|udp_output|{

            if udp_output.player_index == remote_peer.get_player_index() {
                info!("Setting remote peer: {:?}", remote_peer);
                udp_output.remote_peer = Some(remote_peer);
            }
        }).unwrap();
    }
}