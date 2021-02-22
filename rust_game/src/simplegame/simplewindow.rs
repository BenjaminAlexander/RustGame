use opengl_graphics::{GlGraphics, OpenGL};
use crate::gamemanager::RenderReceiver;
use crate::simplegame::{SimpleState, SimpleInput, SimpleInputEventHandler, SimpleInputEvent, STEP_DURATION};
use crate::client::Core;
use crate::threading::{Sender, Thread, Consumer};
use piston::{RenderArgs, WindowSettings, Events, EventSettings, RenderEvent, Event};
use piston::input::Input as PistonInput;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use log::info;

pub struct SimpleWindow {
    render_receiver: RenderReceiver<SimpleState, SimpleInput>,
    client_core_sender_option: Option<Sender<Core<SimpleState, SimpleInput, SimpleState, SimpleInputEventHandler, SimpleInputEvent>>>
}

impl SimpleWindow {

    pub fn new(render_receiver: RenderReceiver<SimpleState, SimpleInput>,
               client_core_sender_option: Option<Sender<Core<SimpleState, SimpleInput, SimpleState, SimpleInputEventHandler, SimpleInputEvent>>>) -> Self {

        return Self{
            render_receiver,
            client_core_sender_option
        }
    }

    pub fn run(mut self) -> () {

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        //EventLoopExtWindows::new_any_thread
        let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let mut gl = GlGraphics::new(opengl);

        let mut simple_window = SimpleWindow{
            render_receiver: self.render_receiver,
            client_core_sender_option: self.client_core_sender_option
        };


        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut window) {

            if let Some(args) = e.render_args() {
                simple_window.render(&mut gl, &args);

            } else {

                match e {
                    Event::Input(input, _) => {
                        simple_window.input(input)
                    },
                    _ => {}
                }
            }
        }
        info!("Done");
        return ();
    }

    fn render(&mut self, gl_graphics: &mut GlGraphics, args: &RenderArgs) {

        let step_message = self.render_receiver.get_step_message();

        gl_graphics.draw(args.viewport(), |c, gl| {

            const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

            // Clear the screen.
            clear(GREEN, gl);

            if step_message.is_some() {
                let (duration_since_game_start, step_message) = step_message.unwrap();
                //let duration_since_game_start = STEP_DURATION * step_message.get_step_index() as i64;
                step_message.get_state().draw(duration_since_game_start, args, c, gl);
            }

        });
    }

    fn input(&mut self, input: PistonInput) {
        if let Some(core_sender) = self.client_core_sender_option.as_ref() {
            core_sender.accept(SimpleInputEvent::new(input));
        }
    }
}