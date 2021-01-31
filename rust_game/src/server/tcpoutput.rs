use std::net::TcpStream;
use crate::gametime::TimeMessage;
use crate::threading::{ChannelDrivenThread, Consumer, Sender};
use std::io;
use crate::messaging::{ToClientMessage, InputMessage, StateMessage, InitialInformation};
use std::io::Write;
use crate::interface::{Input, State, InputEvent};
use std::marker::PhantomData;

pub struct TcpOutput<StateType, InputType, InputEventType>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

    player_index: usize,
    tcp_stream: TcpStream,
    time_message: Option<TimeMessage>,
    last_state_sequence: Option<usize>,
    input_queue: Vec<InputMessage<InputType>>,
    state_message: Option<StateMessage<StateType>>,
    phantom: PhantomData<InputEventType>
}

impl<StateType, InputType, InputEventType> TcpOutput<StateType, InputType, InputEventType>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

    pub fn new(player_index: usize, tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(TcpOutput{
            player_index,
            tcp_stream: tcp_stream.try_clone()?,
            time_message: None,
            last_state_sequence: None,
            input_queue: Vec::new(),
            state_message: None,
            phantom: PhantomData
        })
    }
}

impl<StateType, InputType, InputEventType> ChannelDrivenThread<()> for TcpOutput<StateType, InputType, InputEventType>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

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

impl<StateType, InputType, InputEventType> Consumer<TimeMessage> for Sender<TcpOutput<StateType, InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |tcp_output|{
            if tcp_output.time_message.is_none() ||
                time_message.is_after(&tcp_output.time_message.clone().unwrap()) {
                tcp_output.time_message = Some(time_message);
            }
        }).unwrap();
    }
}

impl<StateType, InputType, InputEventType> Consumer<InputMessage<InputType>> for Sender<TcpOutput<StateType, InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

    fn accept(&self, input_message: InputMessage<InputType>) {

        self.send(move |tcp_output|{

            if tcp_output.player_index != input_message.get_player_index() &&
                (tcp_output.last_state_sequence.is_none() ||
                tcp_output.last_state_sequence.as_ref().unwrap() < &input_message.get_step()) {
                //insert in reverse sorted order
                match tcp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                    Ok(pos) => tcp_output.input_queue[pos] = input_message,
                    Err(pos) => tcp_output.input_queue.insert(pos, input_message)
                }
            }
        }).unwrap();
    }
}

impl<StateType, InputType, InputEventType> Consumer<StateMessage<StateType>> for Sender<TcpOutput<StateType, InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

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
                                tcp_output.input_queue.pop();
                            }
                        }
                    }
                }
            }
        }).unwrap();
    }
}

impl<StateType, InputType, InputEventType> Sender<TcpOutput<StateType, InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          StateType: State<InputType, InputEventType>,
          InputEventType: InputEvent {

    pub fn send_initial_information(&self, player_count: usize, initial_state: StateType) {
        self.send(move |tcp_output|{

            let initial_information = InitialInformation::<StateType>::new(
                player_count,
                tcp_output.player_index,
                initial_state);

            let message = ToClientMessage::<StateType, InputType>::InitialInformation(initial_information);
            rmp_serde::encode::write(&mut tcp_output.tcp_stream, &message).unwrap();
            tcp_output.tcp_stream.flush().unwrap();

        }).unwrap();
    }
}