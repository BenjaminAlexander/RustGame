use commons::{factory::FactoryTrait, threading::{eventhandling::{EventSenderTrait, EventHandlerSender}, AsyncJoin}};
use crate::{server::{ServerCoreEvent, ServerCore}, gamemanager::{RenderReceiver, RenderReceiverMessage}};
use super::GameFactoryTrait;
use log::error;

pub struct Server<GameFactory: GameFactoryTrait> {
    core_sender: EventHandlerSender<GameFactory::Factory, ServerCoreEvent<GameFactory>>,
    render_receiver_sender_option: Option<<GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>>,
    render_receiver_option: Option<RenderReceiver<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> Server<GameFactory> {

    pub fn new(factory: GameFactory::Factory) -> Result<Self, ()> {

        let server_core_thread_builder = factory.new_thread_builder()
            .name("ServerCore")
            .build_channel_for_event_handler::<ServerCore<GameFactory>>();

        let server_core = ServerCore::<GameFactory>::new(factory.clone(), server_core_thread_builder.get_sender().clone());

        if let Err(error) = server_core_thread_builder.get_sender().send_event(ServerCoreEvent::StartListenerEvent) {
            error!("{:?}", error);
            return Err(());
        }

        let core_sender: <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::Sender<commons::threading::eventhandling::EventOrStopThread<ServerCoreEvent<GameFactory>>> = server_core_thread_builder.spawn_event_handler(server_core, AsyncJoin::log_async_join).unwrap();

        let (render_receiver_sender, render_receiver) = RenderReceiver::<GameFactory>::new(factory.clone());


        return Ok(Self {
            core_sender,
            render_receiver_sender_option: Some(render_receiver_sender),
            render_receiver_option: Some(render_receiver)
        });
    }

    pub fn start_game(&mut self) -> Result<(), ()> {

        match self.render_receiver_sender_option.take() {
            Some(render_receiver_sender) => {

                match self.core_sender.send_event(ServerCoreEvent::StartGameEvent(render_receiver_sender)) {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        error!("An error occurred when signaling the server to start: {:?}", error);
                        return Err(());
                    },
                }

            },
            None => {
                error!("The server has already been started");
                return Err(());
            },
        }
    }

    pub fn take_render_receiver(&mut self) -> Option<RenderReceiver<GameFactory>> {
        return self.render_receiver_option.take();
    }
    
}