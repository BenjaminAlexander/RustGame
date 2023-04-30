use std::net::{UdpSocket, SocketAddrV4, SocketAddr};
use commons::time::{TimeDuration, TimeValue};
use crate::gametime::TimeReceived;
use crate::messaging::{ToClientMessageUDP, MessageFragment, FragmentAssembler};
use commons::threading::{eventhandling, listener};
use crate::interface::GameFactoryTrait;
use std::io;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use log::{debug, error, warn};
use commons::factory::FactoryTrait;
use commons::net::UdpReadHandlerTrait;
use crate::gamemanager::ManagerEvent;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::EventSenderTrait;
use commons::threading::listener::{ListenerEventResult, ListenerTrait, ListenMetaData, ListenResult};
use commons::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};
use crate::client::ClientCoreEvent;

pub struct UdpInput<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    fragment_assembler: FragmentAssembler,
    core_sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>,
    manager_sender: eventhandling::Sender<GameFactory::Factory, ManagerEvent<GameFactory::Game>>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
    time_of_last_server_input_receive: TimeValue,
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {

    pub fn new(
        factory: GameFactory::Factory,
        core_sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>,
        manager_sender: eventhandling::Sender<GameFactory::Factory, ManagerEvent<GameFactory::Game>>) -> io::Result<Self> {

        return Ok(Self{
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(5),
            core_sender,
            manager_sender,

            //metrics
            time_of_last_state_receive: factory.now(),
            time_of_last_input_receive: factory.now(),
            time_of_last_server_input_receive: factory.now(),
            factory,
        });
    }
}

impl<GameFactory: GameFactoryTrait> UdpReadHandlerTrait for UdpInput<GameFactory> {

    fn on_read(&mut self, peer_addr: SocketAddr, buff: &[u8]) -> ControlFlow<()> {

        let fragment = MessageFragment::from_vec(buff.to_vec());

        if let Some(message_buf) = self.fragment_assembler.add_fragment(&self.factory, fragment) {

            match rmp_serde::from_read_ref(&message_buf) {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    return self.handle_received_message(message);
                }
                Err(error) => {
                    error!("Error: {:?}", error);
                }
            }
        }

        return Continue(());
    }
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {

    fn handle_received_message(&mut self, value: ToClientMessageUDP<GameFactory::Game>) -> ControlFlow<()> {

        let time_received = self.factory.now();

        match value {
            ToClientMessageUDP::TimeMessage(time_message) => {
                //info!("Time message: {:?}", time_message.get_step());
                self.core_sender.send_event(ClientCoreEvent::RemoteTimeMessageEvent(TimeReceived::new(time_received, time_message))).unwrap();
            }
            ToClientMessageUDP::InputMessage(input_message) => {
                //TODO: ignore input messages from this player
                //info!("Input message: {:?}", input_message.get_step());
                self.time_of_last_input_receive = time_received;
                self.manager_sender.send_event(ManagerEvent::InputEvent(input_message.clone())).unwrap();
            }
            ToClientMessageUDP::ServerInputMessage(server_input_message) => {
                //info!("Server Input message: {:?}", server_input_message.get_step());
                self.time_of_last_server_input_receive = time_received;
                self.manager_sender.send_event(ManagerEvent::ServerInputEvent(server_input_message)).unwrap();
            }
            ToClientMessageUDP::StateMessage(state_message) => {
                //info!("State message: {:?}", state_message.get_sequence());

                let duration_since_last_state = time_received.duration_since(&self.time_of_last_state_receive);
                if duration_since_last_state > TimeDuration::one_second() {

                    //TODO: this should probably be a warn
                    debug!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                            duration_since_last_state, time_received, self.time_of_last_state_receive);
                }

                self.time_of_last_state_receive = time_received;
                self.manager_sender.send_event(ManagerEvent::StateEvent(state_message)).unwrap();
            }
        };

        return Continue(());
    }
}
