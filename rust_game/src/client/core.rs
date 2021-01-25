use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream};
use std::str::FromStr;
use crate::gametime::{TimeDuration, GameTimer};
use crate::threading::{ChannelThread, Sender, ChannelDrivenThread, Consumer};
use crate::client::tcpinput::TcpInput;
use crate::interface::{Input, State};
use std::marker::PhantomData;
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{StateMessage, InitialInformation};
use crate::gamemanager::Manager;

pub struct Core<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    server_ip: String,
    port: u16,
    step_duration: TimeDuration,
    grace_period: TimeDuration,
    clock_average_size: usize,
    phantom: PhantomData<InputType>,
    state_phantom: PhantomData<StateType>
}

impl<StateType, InputType> Core<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn new(server_ip: &str,
               port: u16,
               step_duration: TimeDuration,
               grace_period: TimeDuration,
               clock_average_size: usize) -> Self {

        Core{server_ip: server_ip.to_string(),
            port,
            step_duration,
            grace_period,
            clock_average_size,
            phantom: PhantomData,
            state_phantom: PhantomData
        }
    }
}

impl<StateType, InputType> ChannelDrivenThread<()> for Core<StateType, InputType>
    where InputType: Input,
          StateType: State<InputType> {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Sender<Core<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    pub fn connect(&self) {
        let core_sender = self.clone();
        self.send(|core|{
            let ip_addr_v4 = Ipv4Addr::from_str(core.server_ip.as_str()).unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, core.port);
            let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
            let tcp_stream = TcpStream::connect(socket_addr).unwrap();

            let (manager_sender, manager_builder) = Manager::<StateType, InputType>::new(core.grace_period).build();
            let (game_timer_sender, game_timer_builder) = GameTimer::new(core.step_duration, core.clock_average_size).build();
            let (tcp_input_sender, tcp_input_builder) = TcpInput::<StateType, InputType>::new(&tcp_stream).unwrap().build();
            let (tcp_output_sender, tcp_output_builder) = TcpOutput::<InputType>::new(&tcp_stream).unwrap().build();

            tcp_input_sender.add_time_message_consumer(game_timer_sender.clone()).unwrap();
            tcp_input_sender.add_initial_information_message_consumer(manager_sender.clone());
            tcp_input_sender.add_input_message_consumer(manager_sender.clone());
            tcp_input_sender.add_state_message_consumer(manager_sender.clone());

            game_timer_sender.add_timer_message_consumer(manager_sender.clone());

            let _manager_join_handle = manager_builder.name("ClientManager").start().unwrap();
            let _tcp_input_join_handle = tcp_input_builder.name("ClientTcpInput").start().unwrap();
            let _game_timer_join_handle = game_timer_builder.name("ClientGameTimer").start().unwrap();

        }).unwrap();
    }
}

