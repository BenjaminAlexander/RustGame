use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream};
use std::str::FromStr;
use crate::gametime::{TimeDuration, GameTimer};
use crate::threading::{ChannelThread, Sender, ChannelDrivenThread};
use crate::client::tcpinput::TcpInput;
use crate::interface::Input;
use std::marker::PhantomData;

pub struct Core<InputType>
    where InputType: Input {

    server_ip: String,
    port: u16,
    step_duration: TimeDuration,
    clock_average_size: usize,
    phantom: PhantomData<InputType>
}

impl<InputType> Core<InputType>
    where InputType: Input {

    pub fn new(server_ip: &str,
               port: u16,
               step_duration: TimeDuration,
               clock_average_size: usize) -> Self {

        Core{server_ip: server_ip.to_string(),
            port,
            step_duration,
            clock_average_size,
            phantom: PhantomData
        }
    }
}

impl<InputType> ChannelDrivenThread<()> for Core<InputType>
    where InputType: Input {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<InputType> Sender<Core<InputType>>
    where InputType: Input {

    pub fn connect(&self) {
        self.send(|core|{
            let ip_addr_v4 = Ipv4Addr::from_str(core.server_ip.as_str()).unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, core.port);
            let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
            let tcp_stream = TcpStream::connect(socket_addr).unwrap();

            let (game_timer_sender, game_timer_builder) = GameTimer::new(core.step_duration, core.clock_average_size).build();
            let (tcp_input_sender, tcp_input_builder) = TcpInput::<InputType>::new(&tcp_stream).unwrap().build();

            tcp_input_sender.add_time_message_consumer(game_timer_sender).unwrap();

            let _tcp_input_join_handle = tcp_input_builder.name("ClientTcpInput").start().unwrap();
            let _game_timer_join_handle = game_timer_builder.name("ClientGameTimer").start().unwrap();
        }).unwrap();
    }
}