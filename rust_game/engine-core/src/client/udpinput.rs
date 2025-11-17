use crate::client::ClientCoreEvent;
use crate::gamemanager::ManagerEvent;
use crate::gametime::TimeReceived;
use crate::interface::GameFactoryTrait;
use crate::messaging::{
    FragmentAssembler,
    MessageFragment,
    ToClientMessageUDP,
};
use commons::real_time::net::udp::UdpReadHandlerTrait;
use commons::real_time::{
    EventSender,
    FactoryTrait,
};
use commons::time::{
    TimeDuration,
    TimeValue,
};
use log::{
    debug,
    error,
    warn,
};
use std::io;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    fragment_assembler: FragmentAssembler,
    core_sender: EventSender<ClientCoreEvent<GameFactory>>,
    manager_sender: EventSender<ManagerEvent<GameFactory::Game>>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
    time_of_last_server_input_receive: TimeValue,
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        core_sender: EventSender<ClientCoreEvent<GameFactory>>,
        manager_sender: EventSender<ManagerEvent<GameFactory::Game>>,
    ) -> io::Result<Self> {
        return Ok(Self {
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(&factory, 5),
            core_sender,
            manager_sender,

            //metrics
            time_of_last_state_receive: factory.get_time_source().now(),
            time_of_last_input_receive: factory.get_time_source().now(),
            time_of_last_server_input_receive: factory.get_time_source().now(),
            factory,
        });
    }
}

impl<GameFactory: GameFactoryTrait> UdpReadHandlerTrait for UdpInput<GameFactory> {
    fn on_read(&mut self, peer_addr: SocketAddr, buff: &[u8]) -> ControlFlow<()> {
        let fragment = MessageFragment::from_vec(buff.to_vec());

        if let Some(message_buf) = self
            .fragment_assembler
            .add_fragment(fragment)
        {
            match rmp_serde::from_slice(&message_buf) {
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

        return ControlFlow::Continue(());
    }
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {
    fn handle_received_message(
        &mut self,
        value: ToClientMessageUDP<GameFactory::Game>,
    ) -> ControlFlow<()> {
        let time_received = self.factory.get_time_source().now();

        match value {
            ToClientMessageUDP::TimeMessage(time_message) => {
                //info!("Time message: {:?}", time_message.get_step());
                let send_result =
                    self.core_sender
                        .send_event(ClientCoreEvent::RemoteTimeMessageEvent(TimeReceived::new(
                            time_received,
                            time_message,
                        )));

                if send_result.is_err() {
                    warn!("Failed to send TimeMessage to Core");
                    return ControlFlow::Break(());
                }
            }
            ToClientMessageUDP::InputMessage(input_message) => {
                //TODO: ignore input messages from this player
                //info!("Input message: {:?}", input_message.get_step());
                self.time_of_last_input_receive = time_received;
                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::InputEvent(input_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send InputEvent to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            ToClientMessageUDP::ServerInputMessage(server_input_message) => {
                //info!("Server Input message: {:?}", server_input_message.get_step());
                self.time_of_last_server_input_receive = time_received;
                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::ServerInputEvent(server_input_message));

                if send_result.is_err() {
                    warn!("Failed to send ServerInputEvent to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            ToClientMessageUDP::StateMessage(state_message) => {
                //info!("State message: {:?}", state_message.get_sequence());

                let duration_since_last_state =
                    time_received.duration_since(&self.time_of_last_state_receive);
                if duration_since_last_state > TimeDuration::ONE_SECOND {
                    //TODO: this should probably be a warn
                    debug!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                            duration_since_last_state, time_received, self.time_of_last_state_receive);
                }

                self.time_of_last_state_receive = time_received;
                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::StateEvent(state_message));

                if send_result.is_err() {
                    warn!("Failed to send StateMessage to Game Manager");
                    return ControlFlow::Break(());
                }
            }
        };

        return ControlFlow::Continue(());
    }
}
