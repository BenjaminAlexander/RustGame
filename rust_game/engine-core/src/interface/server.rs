use crate::{
    interface::{
        RenderReceiver,
    },
    server::{
        ServerCore,
        ServerCoreEvent,
    },
    GameTrait,
};
use commons::{
    real_time::{
        EventHandlerBuilder,
        EventSender,
        Factory,
    },
    utils::log_error,
};
use log::{
    warn,
};

pub struct Server<Game: GameTrait> {
    core_sender: EventSender<ServerCoreEvent<Game>>,
    render_receiver_option: Option<RenderReceiver<Game>>,
}

impl<Game: GameTrait> Server<Game> {
    pub fn new(factory: Factory) -> Result<Self, ()> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::new(&factory);

        let server_core_thread_builder = EventHandlerBuilder::new(&factory);

        let server_core = ServerCore::new(
                factory, 
                server_core_thread_builder.get_sender().clone(),
                render_receiver_sender.clone()
            )
            .map_err(log_error)?;

        let core_sender = server_core_thread_builder
            .spawn_thread("ServerCore".to_string(), server_core)
            .unwrap();

        return Ok(Self {
            core_sender,
            render_receiver_option: Some(render_receiver),
        });
    }

    pub fn start_game(&mut self) -> Result<(), ()> {
        let send_result = self
            .core_sender
            .send_event(ServerCoreEvent::StartGameEvent);

        if send_result.is_err() {
            warn!("Failed to send ServerCoreEvent to Core");
            return Err(());
        }

        return Ok(());
    }

    pub fn take_render_receiver(&mut self) -> Option<RenderReceiver<Game>> {
        return self.render_receiver_option.take();
    }
}
