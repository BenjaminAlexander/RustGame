use crate::{
    interface::{
        RenderReceiver,
        RenderReceiverMessage,
    },
    server::{
        ServerCore,
        ServerCoreEvent,
    },
    GameTrait,
};
use commons::{real_time::{
    EventHandlerBuilder,
    EventSender,
    Factory,
    Sender,
}, utils::log_error};
use log::{
    error,
    warn,
};

pub struct Server<Game: GameTrait> {
    core_sender: EventSender<ServerCoreEvent<Game>>,
    render_receiver_sender_option: Option<Sender<RenderReceiverMessage<Game>>>,
    render_receiver_option: Option<RenderReceiver<Game>>,
}

impl<Game: GameTrait> Server<Game> {
    pub fn new(factory: Factory) -> Result<Self, ()> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::new(&factory);

        let server_core_thread_builder = EventHandlerBuilder::new(&factory);

        let send_result = server_core_thread_builder
            .get_sender()
            .send_event(ServerCoreEvent::StartListenerEvent);

        if send_result.is_err() {
            warn!("Failed to send StartListenerEvent to Core");
            return Err(());
        }

        let server_core = ServerCore::new(factory, server_core_thread_builder.get_sender().clone()).map_err(log_error)?;

        let core_sender = server_core_thread_builder
            .spawn_thread("ServerCore".to_string(), server_core)
            .unwrap();

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

    pub fn take_render_receiver(&mut self) -> Option<RenderReceiver<Game>> {
        return self.render_receiver_option.take();
    }
}
