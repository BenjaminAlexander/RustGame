use std::net::TcpStream;
use crate::gametime::TimeMessage;
use crate::threading::{ChannelDrivenThread, Consumer, Sender};
use std::io;
use crate::messaging::{ToClientMessage, InputMessage, StateMessage};
use std::io::Write;
use crate::interface::{Input, State};

pub struct TcpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    tcp_stream: TcpStream,
    time_message: Option<TimeMessage>,
    last_state_sequence: Option<usize>,
    input_queue: Vec<InputMessage<InputType>>,
    state_message: Option<StateMessage<StateType>>
}

impl<StateType, InputType> TcpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(TcpOutput{
            tcp_stream: tcp_stream.try_clone()?,
            time_message: None,
            last_state_sequence: None,
            input_queue: Vec::new(),
            state_message: None
        })
    }
}

impl<StateType, InputType> ChannelDrivenThread<()> for TcpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    fn on_none_pending(&mut self) -> Option<()> {

        if self.time_message.is_some() {
            let time_message = self.time_message.unwrap();
            self.time_message = None;

            //TODO: timestamp when the time message is set, then use that info in client side time calc
            let message = ToClientMessage::<StateType, InputType>::TimeMessage(time_message);

            rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
            self.tcp_stream.flush().unwrap();

        } else if self.state_message.is_some() {
            let message = self.state_message.as_ref().unwrap().clone();
            let message = ToClientMessage::<StateType, InputType>::StateMessage(message);
            self.state_message = None;

            rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
            self.tcp_stream.flush().unwrap();

        } else {
            match self.input_queue.pop() {
                None => {},
                Some(input_to_send) => {
                    let message = ToClientMessage::<StateType, InputType>::InputMessage(input_to_send);
                    rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
                    self.tcp_stream.flush().unwrap();
                }
            }
        }

        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Consumer<TimeMessage> for Sender<TcpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |tcp_output|{
            if tcp_output.time_message.is_none() ||
                time_message.is_after(&tcp_output.time_message.clone().unwrap()) {
                tcp_output.time_message = Some(time_message);
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<TcpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State {

    fn accept(&self, input_message: InputMessage<InputType>) {

        self.send(move |tcp_output|{

            if tcp_output.last_state_sequence.is_none() ||
                tcp_output.last_state_sequence.as_ref().unwrap() < &input_message.get_sequence() {
                //insert in reverse sorted order
                match tcp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                    Ok(pos) => tcp_output.input_queue[pos] = input_message,
                    Err(pos) => tcp_output.input_queue.insert(pos, input_message)
                }
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<StateMessage<StateType>> for Sender<TcpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State {

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
                            if last.get_sequence() < tcp_output.last_state_sequence.unwrap() {
                                tcp_output.input_queue.pop();
                            }
                        }
                    }
                }
            }
        }).unwrap();
    }
}