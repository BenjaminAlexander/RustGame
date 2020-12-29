use std::net::TcpStream;
use log::info;
use crate::threading::{Consumer, Sender, ChannelThread, ChannelDrivenThread};
use crate::server::tcpinput::{TcpInput, TestConsumer};
use serde::export::PhantomData;
use crate::interface::{State, Input};

pub struct Core<StateType, InputType>
    where StateType: State,
          InputType: Input {
    tcp_inputs: Vec<Sender<TcpInput<InputType>>>,
    phantom: PhantomData<StateType>
}

impl<StateType, InputType> ChannelDrivenThread for Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn on_none_pending(&mut self) {
        info!("on_none_pending.");
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

            sender.add_input_consumer(TestConsumer{});
            thread_builder.name("TcpInput".to_string()).start().unwrap();

            core.tcp_inputs.push(sender);
        });
    }
}

