use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream};
use std::str::FromStr;
use crate::gametime::{TimeDuration, GameTimer, TimeMessage};
use crate::threading::{ChannelThread, Sender, ChannelDrivenThread, Consumer};
use crate::client::tcpinput::TcpInput;
use crate::interface::{Input, State, InputEvent};
use std::marker::PhantomData;
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{StateMessage, InitialInformation};
use crate::gamemanager::Manager;

pub struct Core<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    server_ip: String,
    port: u16,
    step_duration: TimeDuration,
    grace_period: TimeDuration,
    clock_average_size: usize,
    manager_sender: Option<Sender<Manager<StateType, InputType, InputEventType>>>,
}

impl<StateType, InputType, InputEventType> Core<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

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
            manager_sender: None
        }
    }
}

impl<StateType, InputType, InputEventType> ChannelDrivenThread<()> for Core<StateType, InputType, InputEventType>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType, InputEventType> Sender<Core<StateType, InputType, InputEventType>>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn connect(&self) {
        let core_sender = self.clone();
        self.send(move |core|{
            let ip_addr_v4 = Ipv4Addr::from_str(core.server_ip.as_str()).unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, core.port);
            let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
            let tcp_stream = TcpStream::connect(socket_addr).unwrap();

            let (manager_sender, manager_builder) = Manager::<StateType, InputType, InputEventType>::new(core.grace_period).build();
            let (game_timer_sender, game_timer_builder) = GameTimer::new(core.step_duration, core.clock_average_size).build();
            let (tcp_input_sender, tcp_input_builder) = TcpInput::<StateType, InputType, InputEventType>::new(&tcp_stream).unwrap().build();
            let (tcp_output_sender, tcp_output_builder) = TcpOutput::<InputType, InputEventType>::new(&tcp_stream).unwrap().build();

            tcp_input_sender.add_time_message_consumer(game_timer_sender.clone()).unwrap();
            tcp_input_sender.add_initial_information_message_consumer(manager_sender.clone());
            tcp_input_sender.add_input_message_consumer(manager_sender.clone());
            tcp_input_sender.add_state_message_consumer(manager_sender.clone());

            game_timer_sender.add_timer_message_consumer(core_sender.clone());

            let _manager_join_handle = manager_builder.name("ClientManager").start().unwrap();
            let _tcp_input_join_handle = tcp_input_builder.name("ClientTcpInput").start().unwrap();
            let _game_timer_join_handle = game_timer_builder.name("ClientGameTimer").start().unwrap();

            core.manager_sender = Some(manager_sender);

        }).unwrap();
    }
}

impl<StateType, InputType, InputEventType> Consumer<TimeMessage> for Sender<Core<StateType, InputType, InputEventType>>
    where StateType: State<InputType, InputEventType>,
          InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |core|{
            if core.manager_sender.is_some() {
                let client_drop_time = time_message.get_scheduled_time().subtract(core.grace_period * 2);
                let drop_step = time_message.get_step_from_actual_time(client_drop_time);
                let core_sender = core.manager_sender.as_ref().unwrap();

                core_sender.drop_steps_before(drop_step);
                core_sender.set_requested_step(time_message.get_step());
            }

        }).unwrap();
    }
}

