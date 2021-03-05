use log::{trace, info, warn};
use std::net::TcpStream;
use crate::gametime::{TimeMessage, TimeDuration, TimeValue};
use crate::threading::{ChannelDrivenThread, Consumer, Sender, ChannelThread, Receiver};
use std::io;
use crate::messaging::{ToClientMessageTCP, InputMessage, StateMessage, InitialInformation};
use std::io::Write;
use crate::interface::{Input, State, InputEvent};
use std::marker::PhantomData;
use crate::server::ServerConfig;

pub struct TcpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    player_index: usize,
    tcp_stream: TcpStream,
    state_phantom: PhantomData<StateType>,
    input_phantom: PhantomData<InputType>
}

impl<StateType, InputType> TcpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    pub fn new(player_index: usize,
               tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(TcpOutput{
            player_index,
            tcp_stream: tcp_stream.try_clone()?,
            state_phantom: PhantomData,
            input_phantom: PhantomData
        })
    }
}

impl<StateType, InputType> ChannelThread<()> for TcpOutput<StateType, InputType>
    where InputType: Input,
          StateType: State {

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

            receiver.try_iter(&mut self);
        }
    }
}

impl<StateType, InputType> Sender<TcpOutput<StateType, InputType>>
    where InputType: Input,
          StateType: State {

    pub fn send_initial_information(&self, server_config: ServerConfig, player_count: usize, initial_state: StateType) {
        self.send(move |tcp_output|{

            let initial_information = InitialInformation::<StateType>::new(
                server_config,
                player_count,
                tcp_output.player_index,
                initial_state
            );

            let message = ToClientMessageTCP::<StateType>::InitialInformation(initial_information);
            rmp_serde::encode::write(&mut tcp_output.tcp_stream, &message).unwrap();
            tcp_output.tcp_stream.flush().unwrap();

        }).unwrap();
    }
}