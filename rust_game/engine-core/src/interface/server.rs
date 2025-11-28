use crate::{
    GameTrait, interface::RenderReceiver, server::{
        ServerCore, ServerCoreBuilder,
    }
};
use commons::{
    real_time::{
        Factory,
    },
};

pub struct Server<Game: GameTrait> {
    server_core: ServerCore<Game>,
    render_receiver_option: Option<RenderReceiver<Game>>,
}

impl<Game: GameTrait> Server<Game> {
    pub fn new(factory: Factory) -> Result<Self, ()> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::new(&factory);

        let server_core = ServerCoreBuilder::new(factory.clone())
            .spawn_thread(render_receiver_sender.clone())
            .unwrap();

        return Ok(Self {
            server_core,
            render_receiver_option: Some(render_receiver),
        });
    }

    pub fn start_game(&self) -> Result<(), ()> {
        self.server_core.start_game()
    }

    pub fn take_render_receiver(&mut self) -> Option<RenderReceiver<Game>> {
        return self.render_receiver_option.take();
    }
}
