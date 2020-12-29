use std::net::TcpStream;
use std::thread::{Builder, JoinHandle};
use log::{info, warn, error};
use crate::threading::{Consumer, Sender, ChannelThread, ChannelDrivenThread};
use crate::threading;
use crate::server::tcpinput::{TcpInput, TestConsumer};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use serde::export::PhantomData;
use crate::threading::Thread;
use crate::interface::{State, Input};

pub struct Core<StateType, InputType>
    where StateType: State,
          InputType: Input {
    tcpInputs: Vec<Sender<TcpInput<InputType>>>,
    phantom: PhantomData<StateType>
}

impl<StateType, InputType> ChannelDrivenThread for Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn onNonePending(&mut self) {
        info!("onNonePending.");
    }
}

impl<StateType, InputType> Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    pub fn new() -> Self {
        Core { tcpInputs: Vec::new(), phantom: PhantomData }
    }
}

impl<StateType, InputType> Consumer<TcpStream> for Sender<Core<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    fn accept(&self, tcpStream: TcpStream) {
        self.send(|core|{

            let (sender, threadBuilder) = TcpInput::new(tcpStream).build();

            sender.add_input_consumer(TestConsumer{});
            let joinHandle = threadBuilder.name("TcpInput".to_string()).start().unwrap();

            //let (sender, joinHandle) = tcpInput.start();

            core.tcpInputs.push(sender);
        });
    }
}
