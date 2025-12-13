use crate::simplegameimpl::SimpleGameImpl;
use crate::simpleinputevent::SimpleInputEvent;
use engine_core::{
    Client,
    StateReceiver,
    StateReceiverError,
};
use glutin_window::GlutinWindow as Window;
use graphics::*;
use log::info;
use opengl_graphics::{
    GlGraphics,
    OpenGL,
};
use piston::input::Input as PistonInput;
use piston::{
    Event,
    EventSettings,
    Events,
    Motion,
    RenderArgs,
    RenderEvent,
    WindowSettings,
};

pub struct SimpleWindow {
    window_name: String,
    render_receiver: StateReceiver<SimpleGameImpl>,
    //TODO: don't expose eventhandling, sender or ClientCore, or ClientCoreEvent, or GameFactoryTrait, or RealGameFactory
    client_option: Option<Client<SimpleGameImpl>>,
    mouse_position: [f64; 2],
}

impl SimpleWindow {
    pub fn new(
        window_name: String,
        render_receiver: StateReceiver<SimpleGameImpl>,
        client_option: Option<Client<SimpleGameImpl>>,
    ) -> Self {
        return Self {
            window_name,
            render_receiver,
            client_option,
            mouse_position: [0.0, 0.0],
        };
    }

    pub fn run(mut self) -> () {
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

        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut window) {
            if let Some(args) = e.render_args() {
                self.render(&mut gl, &args);
            } else {
                match e {
                    Event::Input(input, _) => self.input(input),
                    _ => {}
                }
            }
        }
        info!("Done");
        return ();
    }

    fn render(&mut self, gl_graphics: &mut GlGraphics, args: &RenderArgs) {
        let mouse_position = &self.mouse_position;

        let result = self.render_receiver.get_step_message();
        let current_states = match result {
            Ok(current_states) => Some(current_states),
            Err(StateReceiverError::StateNoYetAvailable) => None,
            Err(StateReceiverError::Disconnected) => panic!("Receiver Disconnected"),
        };

        gl_graphics.draw(args.viewport(), |context, gl| {
            const GREEN: [f32; 4] = [0.7, 0.7, 0.3, 1.0];

            // Clear the screen.
            clear(GREEN, gl);

            if let Some(current_states) = &current_states {
                let now = current_states.time_source.now();
                let duration_since_game_start =
                    now.duration_since(current_states.start_time.get_time_value());

                //let duration_since_game_start = STEP_DURATION * step_message.get_step_index() as i64;
                current_states.next_frame.get_state().draw(
                    current_states.initial_information,
                    duration_since_game_start,
                    context,
                    gl,
                );
            }

            draw_mouse(mouse_position, context, gl)
        });
    }

    fn input(&mut self, input: PistonInput) {
        //TODO: track local mouse position here
        match input {
            PistonInput::Move(Motion::MouseCursor(ref position)) => {
                self.mouse_position = *position;
            }
            _ => {}
        }

        if let Some(client) = self.client_option.as_ref() {
            client
                .send_client_input_event(SimpleInputEvent::new(input))
                .unwrap();
        }
    }
}

fn draw_mouse(mouse_position: &[f64; 2], context: Context, gl: &mut GlGraphics) {
    const MOUSE_COLOR: [f32; 4] = [0.0, 1.0, 1.0, 1.0];

    let square = rectangle::square(0.0, 0.0, 10.0);

    let transform = context
        .transform
        .trans(mouse_position[0], mouse_position[1])
        .trans(-5.0, -5.0);

    rectangle(MOUSE_COLOR, square, transform, gl);
}
