use std::{thread, time, io};
use log::info;
use crate::messaging::*;
use crate::simplegame::{Vector2, SimpleInput, SimpleState, SimpleInputEvent, STEP_DURATION, SimpleInputEventHandler, SimpleWindow};
use crate::threading::{ChannelThread, Consumer, Thread};
use crate::gametime::TimeDuration;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Event, MouseRelativeEvent, MouseCursorEvent};
use crate::gamemanager::RenderReceiver;
use graphics::*;
use piston::input::Input as PistonInput;
use std::io::Read;
use std::collections::hash_map::{DefaultHasher, RandomState};
use std::hash::{Hash, Hasher, BuildHasher};

mod simplegame;
mod messaging;
mod server;
mod logging;
mod threading;
mod interface;
mod gametime;
mod util;
mod client;
mod gamemanager;

pub fn main() {



    logging::init_logging();

    let args: Vec<String> = std::env::args().collect();

    info!("args: {:?}", args);

    let mut run_client = true;
    let mut run_server = true;

    if args.len() == 2 {
        if args[1].eq("-s") {
            run_client = false;

        } else if args[1].eq("-c") {
            run_server = false;
        }
    }

    let mut server_core_sender_option = None;
    let mut render_receiver_option = None;
    let mut unused_render_receiver_option = None;
    let mut client_core_sender_option = None;

    if run_server {
        let server_core  = server::Core::<SimpleState, SimpleInput, SimpleState>::new(
            3456,
            3457,
            STEP_DURATION,
            TimeDuration::from_millis(500),
            TimeDuration::from_millis(1000)
        );

        let (server_core_sender, server_core_builder) = server_core.build();

        server_core_sender.start_listener();
        server_core_builder.name("ServerCore").start().unwrap();

        server_core_sender_option = Some(server_core_sender);
    }

    if run_client {

        let client_core = client::Core::<SimpleState, SimpleInput, SimpleState, SimpleInputEventHandler, SimpleInputEvent>::new(
            "127.0.0.1",
            3456,
            3457,
            STEP_DURATION,
            TimeDuration::from_millis(500),
            50);

        let (client_core_sender, client_core_builder) = client_core.build();

        render_receiver_option = Some(client_core_sender.connect());
        client_core_sender_option = Some(client_core_sender);
        client_core_builder.name("ClientCore").start().unwrap();

        let millis = time::Duration::from_millis(1000);
        thread::sleep(millis);
    }

    if run_server {

        if !run_client {
            info!("Hit enter to start the game.");
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.read_line(&mut line);

            info!("line: {:?}", line);
        }

        unused_render_receiver_option = Some(server_core_sender_option.as_ref().unwrap().start_game());

        if !run_client {
            let tmp = unused_render_receiver_option;
            unused_render_receiver_option = render_receiver_option;
            render_receiver_option = tmp;
        }
    }


    let client_window = SimpleWindow::new(render_receiver_option.unwrap(), client_core_sender_option);
    client_window.run();
}
