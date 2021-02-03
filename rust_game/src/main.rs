use std::{thread, time};
use log::info;
use crate::messaging::*;
use crate::simplegame::{Vector2, SimpleInput, SimpleState};
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

    let server_core  = server::Core::<SimpleState, SimpleInput, Vector2>::new(3456, TimeDuration::from_millis(50), TimeDuration::from_millis(500));
    let (server_core_sender, server_core_builder) = server_core.build();

    server_core_sender.start_listener();
    server_core_builder.name("ServerCore").start().unwrap();

    let client_core = client::Core::<SimpleState, SimpleInput, Vector2>::new(
        "127.0.0.1",
        3456,
        TimeDuration::from_millis(50),
        TimeDuration::from_millis(500),
        50);

    let (client_core_sender, client_core_builder) = client_core.build();

    let mut render_receiver = client_core_sender.connect();
    client_core_builder.name("ClientCore").start().unwrap();

    let millis = time::Duration::from_millis(1000);
    thread::sleep(millis);

    server_core_sender.start_game(SimpleState::new(1));

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
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(args) = e.mouse_cursor_args() {
            //info!("args: {:?}", args);
            client_core_sender.accept(Vector2::new(args[0] as f32, args[1] as f32));
        }
    }


    let ten_millis = time::Duration::from_millis(10000);
    thread::sleep(ten_millis);

}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    render_receiver: RenderReceiver<SimpleState, SimpleInput, Vector2>,  // Rotation for the square.
}

impl App {
    fn render(&mut self, args: &RenderArgs) {

        let step_message = self.render_receiver.get_step_message();
        let mut x = 0 as f64;
        let mut y = 0 as f64;

        if step_message.is_some() {
            let (sx, sy) = step_message.unwrap().get_state().vectors[0].get();

            //info!("state: {:?}, {:?}", x, y);
            x = (sx as f64 / args.draw_size[0] as f64) * args.window_size[0];
            y = (sy as f64 / args.draw_size[1] as f64) * args.window_size[1];
        }

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);


        //info!("state: {:?}, {:?}", x, y);
        //info!("window_size: {:?}", args.window_size);
        //info!("draw_size: {:?}", args.draw_size);


        let rotation = 0 as f64;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }
}