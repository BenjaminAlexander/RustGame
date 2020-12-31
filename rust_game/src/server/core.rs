use std::net::TcpStream;

use log::info;
use serde::export::PhantomData;

use crate::interface::{Input, State};
use crate::server::tcpinput::{TcpInput, TestConsumer};
use crate::threading::{ChannelDrivenThread, ChannelThread, Consumer, Sender};

pub struct Core<StateType, InputType>
    where StateType: State,
          InputType: Input {
    tcp_inputs: Vec<Sender<TcpInput<InputType>>>,
    phantom: PhantomData<StateType>
}

impl<StateType, InputType> ChannelDrivenThread<()> for Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn on_none_pending(&mut self) -> Option<()> {
        info!("on_none_pending.");
        None
    }
}

impl<StateType, InputType> Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    pub fn new() -> Self {
        Core { tcp_inputs: Vec::new(), phantom: PhantomData }
    }
}

impl<StateType, InputType> Consumer<TcpStream> for Sender<Core<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    fn accept(&self, tcp_stream: TcpStream) {
        self.send(|core|{

            let (sender, thread_builder) = TcpInput::new(tcp_stream).build();

            sender.add_input_consumer(TestConsumer{}).unwrap();
            thread_builder.name("TcpInput").start().unwrap();

            core.tcp_inputs.push(sender);
        }).unwrap();
    }
}

