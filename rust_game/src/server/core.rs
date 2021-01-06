use std::net::TcpStream;

use log::info;

use crate::interface::{Input, State};
use crate::server::tcpinput::TcpInput;
use crate::threading::{ChannelDrivenThread, ChannelThread, Consumer, Sender};
use crate::server::TcpListenerThread;
use crate::server::tcpoutput::TcpOutput;
use crate::gametime::{GameTimer, TimeDuration};

//TODO: route game timer and player inputs through the core to
// get synchronous enforcement of the grace period

pub struct Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    game_is_started: bool,
    port: u16,
    step_duration: TimeDuration,
    tcp_inputs: Vec<Sender<TcpInput<InputType>>>,
    tcp_outputs: Vec<Sender<TcpOutput<StateType, InputType>>>
}

impl<StateType, InputType> ChannelDrivenThread<()> for Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Core<StateType, InputType>
    where StateType: State,
          InputType: Input {

    pub fn new(port: u16, step_duration: TimeDuration) -> Self {
        Self {
            game_is_started: false,
            port,
            step_duration,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new()
        }
    }
}

impl<StateType, InputType> Sender<Core<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    pub fn start_listener(&self) {
        let clone = self.clone();

        self.send(|core| {
            let (listener_sender, listener_builder) = TcpListenerThread::new(core.port).build();
            listener_sender.set_consumer(clone).unwrap();
            listener_builder.name("ServerTcpListener").start().unwrap();
        }).unwrap();
    }

    pub fn start_game(&self) {
        self.send(|core| {
            if !core.game_is_started {
                core.game_is_started = true;

                let (timer_sender, timer_builder) = GameTimer::new(core.step_duration, 0).build();
                timer_sender.start().unwrap();

                for tcp_output in core.tcp_outputs.iter() {
                    timer_sender.add_timer_message_consumer(tcp_output.clone());
                }

                timer_builder.name("ServerTimer").start().unwrap();
            }
        }).unwrap();
    }

}

impl<StateType, InputType> Consumer<TcpStream> for Sender<Core<StateType, InputType>>
    where StateType: State,
          InputType: Input {

    fn accept(&self, tcp_stream: TcpStream) {
        self.send(move |core|{
            if !core.game_is_started {
                let (in_sender, in_thread_builder) = TcpInput::new(&tcp_stream).unwrap().build();
                in_thread_builder.name("ServerTcpInput").start().unwrap();
                core.tcp_inputs.push(in_sender);

                let (out_sender, out_thread_builder) = TcpOutput::new(&tcp_stream).unwrap().build();
                out_thread_builder.name("ServerTcpOutput").start().unwrap();
                core.tcp_outputs.push(out_sender);

                info!("TcpStream accepted: {:?}", tcp_stream);

            } else {
                info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_stream);
            }
        }).unwrap();
    }
}

