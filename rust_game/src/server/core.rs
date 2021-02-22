use std::net::{TcpStream, Ipv4Addr, SocketAddrV4, UdpSocket};

use log::{warn, trace, info};
use crate::interface::{Input, State, InputEvent, StateUpdate};
use crate::server::tcpinput::TcpInput;
use crate::threading::{ChannelDrivenThread, ChannelThread, Consumer, Sender};
use crate::server::TcpListenerThread;
use crate::server::tcpoutput::TcpOutput;
use crate::gametime::{GameTimer, TimeDuration, TimeMessage};
use crate::gamemanager::{Manager, RenderReceiver};
use crate::messaging::{InputMessage, InitialInformation};
use std::str::FromStr;
use crate::server::udpinput::UdpInput;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::server::udpoutput::UdpOutput;
use crate::server::clientaddress::ClientAddress;

//TODO: route game timer and player inputs through the core to
// get synchronous enforcement of the grace period

pub struct Core<StateType, InputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType> {

    game_is_started: bool,
    tcp_port: u16,
    udp_port: u16,
    step_duration: TimeDuration,
    grace_period: TimeDuration,
    timer_message_period: TimeDuration,
    tcp_inputs: Vec<Sender<TcpInput>>,
    tcp_outputs: Vec<Sender<TcpOutput<StateType, InputType>>>,
    udp_socket: Option<UdpSocket>,
    udp_outputs: Vec<Sender<UdpOutput<StateType, InputType>>>,
    udp_input_sender: Option<Sender<UdpInput<InputType>>>,
    manager_sender: Option<Sender<Manager<StateType, InputType, StateUpdateType>>>,
    drop_steps_before: usize,
}

impl<StateType, InputType, StateUpdateType> ChannelDrivenThread<()> for Core<StateType, InputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType> {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType, StateUpdateType> Core<StateType, InputType, StateUpdateType>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    pub fn new(tcp_port: u16,
               udp_port: u16,
               step_duration: TimeDuration,
               grace_period: TimeDuration,
               timer_message_period: TimeDuration) -> Self {

        Self {
            game_is_started: false,
            tcp_port,
            udp_port,
            step_duration,
            grace_period,
            timer_message_period,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            udp_outputs: Vec::new(),
            drop_steps_before: 0,
            udp_socket: None,
            udp_input_sender: None,
            manager_sender: None,
        }
    }
}

impl<StateType, InputType, StateUpdateType> Sender<Core<StateType, InputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    pub fn start_listener(&self) {
        let clone = self.clone();

        self.send(|core| {
            let ip_addr_v4 = Ipv4Addr::from_str("127.0.0.1").unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, core.udp_port);
            core.udp_socket = Some(UdpSocket::bind(socket_addr_v4).unwrap());

            let udp_input = UdpInput::<InputType>::new(core.udp_socket.as_ref().unwrap()).unwrap();
            let (udp_input_sender, udp_input_builder) = udp_input.build();
            udp_input_builder.name("ServerUdpInput").start().unwrap();
            core.udp_input_sender = Some(udp_input_sender);

            let (listener_sender, listener_builder) = TcpListenerThread::new(core.tcp_port).build();
            listener_sender.set_consumer(clone).unwrap();
            listener_builder.name("ServerTcpListener").start().unwrap();
        }).unwrap();
    }

    pub fn start_game(&self) -> RenderReceiver<StateType, InputType> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::<StateType, InputType>::new();
        let core_sender = self.clone();
        self.send(move |core| {
            if !core.game_is_started {
                core.game_is_started = true;

                let initial_state = StateType::new(core.tcp_outputs.len());

                let (manager_sender, manager_builder) = Manager::<StateType, InputType, StateUpdateType>::new(core.step_duration, core.grace_period).build();
                let (timer_sender, timer_builder) = GameTimer::new(core.step_duration, 0).build();

                timer_sender.add_timer_message_consumer(core_sender.clone());
                timer_sender.add_timer_message_consumer(render_receiver_sender.clone());

                manager_sender.add_requested_step_consumer(render_receiver_sender);

                core.manager_sender = Some(manager_sender.clone());
                manager_sender.drop_steps_before(core.drop_steps_before);

                let server_initial_information = InitialInformation::<StateType>::new(
                    core.tcp_outputs.len(),
                    usize::MAX,
                    initial_state.clone());

                manager_sender.accept(server_initial_information);

                timer_sender.start().unwrap();

                for udp_output in core.udp_outputs.iter() {
                    timer_sender.add_timer_message_consumer(udp_output.clone());

                    manager_sender.add_completed_step_consumer(udp_output.clone());
                }

                for tcp_output in core.tcp_outputs.iter() {
                    tcp_output.send_initial_information(core.tcp_outputs.len(), initial_state.clone());
                }

                core.udp_input_sender.as_ref().unwrap().add_input_consumer(core_sender.clone());

                timer_builder.name("ServerTimer").start().unwrap();
                manager_builder.name("ServerManager").start().unwrap();
            }
        }).unwrap();

        return render_receiver;
    }

}

impl<StateType, InputType, StateUpdateType> Consumer<TcpStream> for Sender<Core<StateType, InputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    fn accept(&self, tcp_stream: TcpStream) {
        self.send(move |core|{
            if !core.game_is_started {
                let player_index = core.tcp_inputs.len();

                let client_address = ClientAddress::new(player_index, tcp_stream.peer_addr().unwrap().ip());

                let (in_sender, in_thread_builder) = TcpInput::new(&tcp_stream).unwrap().build();
                in_thread_builder.name("ServerTcpInput").start().unwrap();
                core.tcp_inputs.push(in_sender);

                let (tcp_out_sender, tcp_out_builder) = TcpOutput::new(
                    player_index,
                    &tcp_stream
                ).unwrap().build();

                let (udp_out_sender, udp_out_builder) = UdpOutput::new(
                    core.timer_message_period.clone(),
                    player_index,
                    core.udp_socket.as_ref().unwrap()
                ).unwrap().build();

                let input_sender = core.udp_input_sender.as_ref().unwrap();
                input_sender.add_remote_peer_consumers(udp_out_sender.clone());
                input_sender.accept(client_address);

                tcp_out_builder.name("ServerTcpOutput").start().unwrap();
                udp_out_builder.name("ServerUdpOutput").start().unwrap();

                core.tcp_outputs.push(tcp_out_sender);
                core.udp_outputs.push(udp_out_sender);

                info!("TcpStream accepted: {:?}", tcp_stream);

            } else {
                info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_stream);
            }
        }).unwrap();
    }
}

impl<StateType, InputType, StateUpdateType> Consumer<TimeMessage> for Sender<Core<StateType, InputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |core|{
            core.drop_steps_before = time_message.get_step_from_actual_time(time_message.get_scheduled_time().subtract(core.grace_period)).ceil() as usize;

            if core.manager_sender.is_some() {
                let manager_sender = core.manager_sender.as_ref().unwrap();
                manager_sender.drop_steps_before(core.drop_steps_before);
                manager_sender.set_requested_step(time_message.get_step() + 1);
            }

        }).unwrap();
    }
}

impl<StateType, InputType, StateUpdateType> Consumer<InputMessage<InputType>> for Sender<Core<StateType, InputType, StateUpdateType>>
    where StateType: State,
          InputType: Input,
          StateUpdateType: StateUpdate<StateType, InputType>{

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |core|{

            if core.drop_steps_before <= input_message.get_step() &&
                core.manager_sender.is_some() {

                core.manager_sender.as_ref().unwrap().accept(input_message.clone());
                for udp_output in core.udp_outputs.iter() {
                    udp_output.accept(input_message.clone());
                }
            }
        }).unwrap();
    }
}