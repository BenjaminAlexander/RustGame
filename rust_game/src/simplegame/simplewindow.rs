use opengl_graphics::{GlGraphics, OpenGL};
use crate::gamemanager::RenderReceiver;
use crate::simplegame::SimpleInputEvent;
use crate::client::ClientCore;
use crate::threading::eventhandling;
use piston::{RenderArgs, WindowSettings, Events, EventSettings, RenderEvent, Event};
use piston::input::Input as PistonInput;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use log::info;
use crate::client::ClientCoreEvent::OnInputEvent;
use crate::simplegame::simplegameimpl::SimpleGameImpl;

pub struct SimpleWindow {
    window_name: String,
    render_receiver: RenderReceiver<SimpleGameImpl>,
    //TODO: don't expose eventhandling, sender or ClientCore, or ClientCoreEvent
    client_core_sender_option: Option<eventhandling::JoinHandle<ClientCore<SimpleGameImpl>>>
}

impl SimpleWindow {

    pub fn new(window_name: String,
               render_receiver: RenderReceiver<SimpleGameImpl>,
               client_core_sender_option: Option<eventhandling::JoinHandle<ClientCore<SimpleGameImpl>>>) -> Self {

        return Self{
            window_name,
            render_receiver,
            client_core_sender_option
        }
    }

    pub fn run(self) -> () {

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        //EventLoopExtWindows::new_any_thread
        let mut window: Window = WindowSettings::new(self.window_name.clone(), [200, 200])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let mut gl = GlGraphics::new(opengl);

        let mut simple_window = SimpleWindow{
            window_name: self.window_name,
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

            const GREEN: [f32; 4] = [0.7, 0.7, 0.3, 1.0];

            // Clear the screen.
            clear(GREEN, gl);

            if step_message.is_some() {
                let (duration_since_game_start, step_message) = step_message.unwrap();
                //let duration_since_game_start = STEP_DURATION * step_message.get_step_index() as i64;
                step_message.draw(duration_since_game_start, args, c, gl);
            }

        });
    }

    fn input(&mut self, input: PistonInput) {
        if let Some(core_join_handle) = self.client_core_sender_option.as_ref() {
            core_join_handle.get_sender().send_event(OnInputEvent(SimpleInputEvent::new(input))).unwrap();
        }
    }
}