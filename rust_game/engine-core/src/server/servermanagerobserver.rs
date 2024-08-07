use crate::gamemanager::{
    ManagerObserverTrait,
    StepMessage,
};
use crate::interface::{
    EventSender,
    GameFactoryTrait,
    RenderReceiverMessage,
};
use crate::messaging::{
    ServerInputMessage,
    StateMessage,
};
use crate::server::udpoutput::UdpOutputEvent;
use commons::factory::FactoryTrait;
use commons::threading::channel::SenderTrait;
use commons::threading::eventhandling::EventSenderTrait;

pub struct ServerManagerObserver<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    udp_outputs: Vec<EventSender<GameFactory, UdpOutputEvent<GameFactory::Game>>>,
    render_receiver_sender:
        <GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>,
}

impl<GameFactory: GameFactoryTrait> ServerManagerObserver<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        udp_outputs: Vec<EventSender<GameFactory, UdpOutputEvent<GameFactory::Game>>>,
        render_receiver_sender: <GameFactory::Factory as FactoryTrait>::Sender<
            RenderReceiverMessage<GameFactory::Game>,
        >,
    ) -> Self {
        return Self {
            factory,
            udp_outputs,
            render_receiver_sender,
        };
    }
}

impl<GameFactory: GameFactoryTrait> ManagerObserverTrait for ServerManagerObserver<GameFactory> {
    type Factory = GameFactory::Factory;
    type Game = GameFactory::Game;

    const IS_SERVER: bool = true;

    fn on_step_message(&self, step_message: StepMessage<GameFactory::Game>) {
        let send_result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(step_message));

        //TODO: handle without panic
        if send_result.is_err() {
            panic!("Failed to send StepMessage to Render Receiver");
        }
    }

    fn on_completed_step(&self, state_message: StateMessage<GameFactory::Game>) {
        for udp_output in self.udp_outputs.iter() {
            let send_result =
                udp_output.send_event(UdpOutputEvent::SendCompletedStep(state_message.clone()));

            //TODO: handle without panic
            if send_result.is_err() {
                panic!("Failed to send CompletedStep to UdpOutput");
            }
        }
    }

    fn on_server_input_message(&self, server_input_message: ServerInputMessage<GameFactory::Game>) {
        for udp_output in self.udp_outputs.iter() {
            let send_result = udp_output.send_event(UdpOutputEvent::SendServerInputMessage(
                server_input_message.clone(),
            ));

            //TODO: handle without panic
            if send_result.is_err() {
                panic!("Failed to send ServerInput to UdpOutput");
            }
        }
    }
}
