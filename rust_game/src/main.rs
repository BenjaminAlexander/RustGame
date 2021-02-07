use std::{thread, time};
use log::info;
use crate::messaging::*;
use crate::simplegame::{Vector2, SimpleInput, SimpleState, SimpleInputEvent, STEP_DURATION, SimpleInputEventHandler};
use crate::threading::{ChannelThread, Consumer};
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

pub fn insert(v: &mut Vec<i32>, val: i32) {

    match v.binary_search_by(|elem| { val.cmp(elem) }) {
        Ok(pos) => v[pos] = val,
        Err(pos) => v.insert(pos, val)
    }
}

pub fn main() {

    logging::init_logging();

    info!("Hello, world!");

    let mut v = Vec::<i32>::new();

    insert(&mut v, 6);
    insert(&mut v, 4);
    insert(&mut v, 5);

    info!("test {:?}", v);
    info!("test {:?}", v.pop().unwrap());
    info!("test {:?}", v);

    let input_message:InputMessage<Vector2> = InputMessage::new(0, 0, Vector2::new(1.0, 12.0));
    let _my_message:ToServerMessage<Vector2> = ToServerMessage::Input(input_message);

    let server_core  = server::Core::<SimpleState, SimpleInput>::new(3456, STEP_DURATION, TimeDuration::from_millis(500));
    let (server_core_sender, server_core_builder) = server_core.build();

    server_core_sender.start_listener();
    server_core_builder.name("ServerCore").start().unwrap();

    let client_core = client::Core::<SimpleState, SimpleInput, SimpleInputEventHandler, SimpleInputEvent>::new(
        "127.0.0.1",
        3456,
        STEP_DURATION,
        TimeDuration::from_millis(500),
        50);

    let (client_core_sender, client_core_builder) = client_core.build();

    let mut render_receiver = client_core_sender.connect();
    client_core_builder.name("ClientCore").start().unwrap();

    let millis = time::Duration::from_millis(1000);
    thread::sleep(millis);

    server_core_sender.start_game();

    // let mut stream = TcpStream::connect("127.0.0.1:3456").unwrap();
    // rmp_serde::encode::write(&mut stream, &my_message).unwrap();
    // stream.flush().unwrap();


    let millis = time::Duration::from_millis(1000);
    thread::sleep(millis);


    //client_core_sender.accept(Vector2::new(3 as f32, 4 as f32));

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let gl = GlGraphics::new(opengl);

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        render_receiver: render_receiver,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {



        if let Some(args) = e.render_args() {
            app.render(&args);
        } else if let Some(args) = e.update_args() {
            app.update(&args);
        } else {

            match e {
                Event::Input(input, _) => {
                    client_core_sender.accept(SimpleInputEvent::new(input));
                }
                _ => {}
            }
        }
    }


    let ten_millis = time::Duration::from_millis(10000);
    thread::sleep(ten_millis);

}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    render_receiver: RenderReceiver<SimpleState, SimpleInput>,  // Rotation for the square.
}

impl App {
    fn render(&mut self, args: &RenderArgs) {

        let step_message = self.render_receiver.get_step_message();

        self.gl.draw(args.viewport(), |c, gl| {

            const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

            // Clear the screen.
            clear(GREEN, gl);

            if step_message.is_some() {
                step_message.unwrap().get_state().draw(args, c, gl);
            }

        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }
}