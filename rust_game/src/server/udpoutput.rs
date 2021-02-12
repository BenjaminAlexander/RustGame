use log::{trace, info, warn};
use crate::interface::{Input, State};
use std::net::{UdpSocket, SocketAddr};
use crate::gametime::{TimeDuration, TimeMessage, TimeValue};
use crate::messaging::{InputMessage, StateMessage, ToClientMessageUDP};
use std::io;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::threading::{ChannelThread, Receiver, Sender, Consumer};
use crate::server::tcpoutput::TcpOutput;

pub struct UdpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    player_index: usize,
    socket: UdpSocket,
    remote_peer: Option<RemoteUdpPeer>,
    time_message_period: TimeDuration,
    last_time_message: Option<TimeMessage>,
    time_message: Option<TimeMessage>,
    last_state_sequence: Option<usize>,
    input_queue: Vec<InputMessage<InputType>>,
    state_message: Option<StateMessage<StateType>>,

    //metrics
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
            time_message: None,
            last_state_sequence: None,
            input_queue: Vec::new(),
            state_message: None,

            //metrics
            time_of_last_state_send: TimeValue::now(),
            time_of_last_input_send: TimeValue::now(),
        })
    }
}

impl<StateType, InputType> ChannelThread<()> for UdpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn run(mut self, receiver: Receiver<Self>) -> () {

        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    info!("Channel closed.");
                    return ();
                }
                _ => {}
            }

            let mut send_another_message = true;
            while send_another_message {
                receiver.try_iter(&mut self);

                if self.time_message.is_some() {

                    let time_message = self.time_message.unwrap();
                    self.time_message = None;
                    self.last_time_message = Some(time_message.clone());

                    //TODO: timestamp when the time message is set, then use that info in client side time calc
                    let message = ToClientMessageUDP::<StateType, InputType>::TimeMessage(time_message);

                    let buf = rmp_serde::to_vec(&message).unwrap();
                    if let Some(remote_peer) = &self.remote_peer {
                        self.socket.send_to(&buf, remote_peer.get_socket_addr()).unwrap();
                    }
                    //info!("time_message");

                } else if self.state_message.is_some() {
                    let message = self.state_message.as_ref().unwrap().clone();
                    let message = ToClientMessageUDP::<StateType, InputType>::StateMessage(message);
                    self.state_message = None;
                    self.time_of_last_state_send = TimeValue::now();

                    let buf = rmp_serde::to_vec(&message).unwrap();
                    if let Some(remote_peer) = &self.remote_peer {
                        self.socket.send_to(&buf, remote_peer.get_socket_addr()).unwrap();
                    }
                    //info!("state_message");

                } else {
                    match self.input_queue.pop() {
                        None => send_another_message = false,
                        Some(input_to_send) => {
                            self.time_of_last_input_send = TimeValue::now();

                            let message = ToClientMessageUDP::<StateType, InputType>::InputMessage(input_to_send);
                            let buf = rmp_serde::to_vec(&message).unwrap();
                            if let Some(remote_peer) = &self.remote_peer {
                                self.socket.send_to(&buf, remote_peer.get_socket_addr()).unwrap();
                            }
                            //info!("input_message");
                        }
                    }
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
}

impl<StateType, InputType> Consumer<TimeMessage> for Sender<UdpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |tcp_output|{
            if tcp_output.time_message.is_none() ||
                time_message.is_after(&tcp_output.time_message.clone().unwrap()) {

                if let Some(last_time_message) = &tcp_output.last_time_message {
                    if time_message.get_scheduled_time().is_after(&last_time_message.get_scheduled_time().add(tcp_output.time_message_period)) {
                        tcp_output.time_message = Some(time_message);
                    }
                } else {
                    tcp_output.time_message = Some(time_message);
                }
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<UdpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, input_message: InputMessage<InputType>) {
        //info!("InputMessage: {:?}", input_message);

        self.send(move |tcp_output|{

            if tcp_output.player_index != input_message.get_player_index() &&
                (tcp_output.last_state_sequence.is_none() ||
                    tcp_output.last_state_sequence.as_ref().unwrap() <= &input_message.get_step()) {
                //insert in reverse sorted order
                match tcp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                    Ok(pos) => tcp_output.input_queue[pos] = input_message,
                    Err(pos) => tcp_output.input_queue.insert(pos, input_message)
                }

                //info!("InputMessage queued. Queue length: {:?}", tcp_output.input_queue.len());

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
        self.send(move |tcp_output|{

            if tcp_output.last_state_sequence.is_none() ||
                tcp_output.last_state_sequence.as_ref().unwrap() <= &state_message.get_sequence() {

                tcp_output.last_state_sequence = Some(state_message.get_sequence());
                tcp_output.state_message = Some(state_message);

                loop {
                    match tcp_output.input_queue.last() {
                        None => break,
                        Some(last) => {
                            if last.get_step() < tcp_output.last_state_sequence.unwrap() {
                                //info!("Popped from que: {:?}, Last state: {:?}", last.get_step(), tcp_output.last_state_sequence.unwrap());
                                tcp_output.input_queue.pop();
                            }
                        }
                    }
                }
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