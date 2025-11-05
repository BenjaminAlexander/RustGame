use super::GameFactoryTrait;
use crate::{
    interface::{
        RenderReceiver,
        RenderReceiverMessage,
    },
    server::{
        ServerCore,
        ServerCoreEvent,
    },
};
use commons::{
    factory::FactoryTrait,
    threading::{
        channel::Sender,
        eventhandling::EventHandlerSender,
        AsyncJoin,
    },
};
use log::{
    error,
    warn,
};

pub struct Server<GameFactory: GameFactoryTrait> {
    core_sender: EventHandlerSender<ServerCoreEvent<GameFactory>>,
    render_receiver_sender_option: Option<Sender<RenderReceiverMessage<GameFactory::Game>>>,
    render_receiver_option: Option<RenderReceiver<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> Server<GameFactory> {
    pub fn new(factory: GameFactory::Factory) -> Result<Self, ()> {
        let server_core_thread_builder = factory
            .new_thread_builder()
            .name("ServerCore")
            .build_channel_for_event_handler::<GameFactory::Factory, ServerCore<GameFactory>>(
            factory.clone(),
        );

        let server_core = ServerCore::<GameFactory>::new(
            factory.clone(),
            server_core_thread_builder.get_sender().clone(),
        );

        let send_result = server_core_thread_builder
            .get_sender()
            .send_event(ServerCoreEvent::StartListenerEvent);

        if send_result.is_err() {
            warn!("Failed to send StartListenerEvent to Core");
            return Err(());
        }

        let core_sender = factory
            .spawn_event_handler(
                server_core_thread_builder,
                server_core,
                AsyncJoin::log_async_join,
            )
            .unwrap();

        let (render_receiver_sender, render_receiver) =
            RenderReceiver::<GameFactory>::new(factory.clone());

        return Ok(Self {
            core_sender,
            render_receiver_sender_option: Some(render_receiver_sender),
            render_receiver_option: Some(render_receiver),
        });
    }

    pub fn start_game(&mut self) -> Result<(), ()> {
        match self.render_receiver_sender_option.take() {
            Some(render_receiver_sender) => {
                let send_result = self
                    .core_sender
                    .send_event(ServerCoreEvent::StartGameEvent(render_receiver_sender));

                if send_result.is_err() {
                    warn!("Failed to send ServerCoreEvent to Core");
                    return Err(());
                }

                return Ok(());
            }
            None => {
                error!("The server has already been started");
                return Err(());
            }
        }
    }

    pub fn take_render_receiver(&mut self) -> Option<RenderReceiver<GameFactory>> {
        return self.render_receiver_option.take();
    }
}
