use std::net::TcpStream;

use log::{warn, trace, info};
use crate::interface::{Input, State};
use crate::server::tcpinput::TcpInput;
use crate::threading::{ChannelDrivenThread, ChannelThread, Consumer, Sender};
use crate::server::TcpListenerThread;
use crate::server::tcpoutput::TcpOutput;
use crate::gametime::{GameTimer, TimeDuration, TimeMessage};
use crate::gamemanager::Manager;
use crate::messaging::{InputMessage, InitialInformation};

//TODO: route game timer and player inputs through the core to
// get synchronous enforcement of the grace period

pub struct Core<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    game_is_started: bool,
    port: u16,
    step_duration: TimeDuration,
    grace_period: TimeDuration,
    tcp_inputs: Vec<Sender<TcpInput<InputType>>>,
    tcp_outputs: Vec<Sender<TcpOutput<StateType, InputType>>>,
    manager_sender: Option<Sender<Manager<StateType, InputType>>>,
    drop_steps_before: usize,
}

impl<StateType, InputType> ChannelDrivenThread<()> for Core<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<StateType, InputType> Core<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn new(port: u16, step_duration: TimeDuration, grace_period: TimeDuration) -> Self {
        Self {
            game_is_started: false,
            port,
            step_duration,
            grace_period,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            drop_steps_before: 0,
            manager_sender: None,
        }
    }
}

impl<StateType, InputType> Sender<Core<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn start_listener(&self) {
        let clone = self.clone();

        self.send(|core| {
            let (listener_sender, listener_builder) = TcpListenerThread::new(core.port).build();
            listener_sender.set_consumer(clone).unwrap();
            listener_builder.name("ServerTcpListener").start().unwrap();
        }).unwrap();
    }

    pub fn start_game(&self, initial_state: StateType) {
        let core_sender = self.clone();
        self.send(move |core| {
            if !core.game_is_started {
                core.game_is_started = true;

                let (manager_sender, manager_builder) = Manager::<StateType, InputType>::new(core.grace_period).build();
                let (timer_sender, timer_builder) = GameTimer::new(core.step_duration, 0).build();

                timer_sender.add_timer_message_consumer(core_sender.clone());

                core.manager_sender = Some(manager_sender.clone());
                manager_sender.drop_steps_before(core.drop_steps_before);

                let server_initial_information = InitialInformation::<StateType>::new(
                    core.tcp_outputs.len(),
                    usize::MAX,
                    initial_state.clone());

                manager_sender.accept(server_initial_information);

                timer_sender.start().unwrap();

                for tcp_output in core.tcp_outputs.iter() {
                    timer_sender.add_timer_message_consumer(tcp_output.clone());
                    manager_sender.add_completed_step_consumer(tcp_output.clone());

                    tcp_output.send_initial_information(core.tcp_outputs.len(), initial_state.clone());
                }

                for tcp_input in core.tcp_inputs.iter() {
                    tcp_input.add_input_consumer(core_sender.clone());
                }

                timer_builder.name("ServerTimer").start().unwrap();
                manager_builder.name("ServerManager").start().unwrap();
            }
        }).unwrap();
    }

}

impl<StateType, InputType> Consumer<TcpStream> for Sender<Core<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    fn accept(&self, tcp_stream: TcpStream) {
        self.send(move |core|{
            if !core.game_is_started {
                let player_index = core.tcp_inputs.len();

                let (in_sender, in_thread_builder) = TcpInput::new(&tcp_stream).unwrap().build();
                in_thread_builder.name("ServerTcpInput").start().unwrap();
                core.tcp_inputs.push(in_sender);

                let (out_sender, out_thread_builder) = TcpOutput::new(player_index, &tcp_stream).unwrap().build();
                out_thread_builder.name("ServerTcpOutput").start().unwrap();
                core.tcp_outputs.push(out_sender);

                info!("TcpStream accepted: {:?}", tcp_stream);

            } else {
                info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_stream);
            }
        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<TimeMessage> for Sender<Core<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |core|{
            core.drop_steps_before = time_message.get_step_from_actual_time(time_message.get_scheduled_time().subtract(core.grace_period));

            if core.manager_sender.is_some() {
                let manager_sender = core.manager_sender.as_ref().unwrap();
                manager_sender.drop_steps_before(core.drop_steps_before);
                manager_sender.set_requested_step(time_message.get_step());
            }

        }).unwrap();
    }
}

impl<StateType, InputType> Consumer<InputMessage<InputType>> for Sender<Core<StateType, InputType>>
    where InputType: Input,
          StateType: State<InputType> {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |core|{

            if core.drop_steps_before <= input_message.get_step() &&
                core.manager_sender.is_some() {

                core.manager_sender.as_ref().unwrap().accept(input_message.clone());
                for tcp_output in core.tcp_outputs.iter() {
                    tcp_output.accept(input_message.clone());
                }
            }
        }).unwrap();
    }
}

