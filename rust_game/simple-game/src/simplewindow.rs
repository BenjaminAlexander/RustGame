use crate::simplegameimpl::SimpleGameImpl;
use crate::simpleinputevent::SimpleInputEvent;
use engine_core::{
    Client,
    RenderReceiver,
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
    render_receiver: RenderReceiver<SimpleGameImpl>,
    //TODO: don't expose eventhandling, sender or ClientCore, or ClientCoreEvent, or GameFactoryTrait, or RealGameFactory
    client_option: Option<Client<SimpleGameImpl>>,
    mouse_position: [f64; 2],
}

impl SimpleWindow {
    pub fn new(
        window_name: String,
        render_receiver: RenderReceiver<SimpleGameImpl>,
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
        let step_message = self.render_receiver.get_step_message();
        let initial_information = self.render_receiver.get_initial_information();

        gl_graphics.draw(args.viewport(), |context, gl| {
            const GREEN: [f32; 4] = [0.7, 0.7, 0.3, 1.0];

            // Clear the screen.
            clear(GREEN, gl);

            if step_message.is_some() && initial_information.is_some() {
                let (duration_since_game_start, step_message) = step_message.unwrap();
                let initial_information = initial_information.as_ref().unwrap();

                //let duration_since_game_start = STEP_DURATION * step_message.get_step_index() as i64;
                step_message.draw(initial_information, duration_since_game_start, context, gl);
            }

            self.draw_mouse(context, gl)
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

    fn draw_mouse(&self, context: Context, gl: &mut GlGraphics) {
        const MOUSE_COLOR: [f32; 4] = [0.0, 1.0, 1.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 10.0);

        let transform = context
            .transform
            .trans(self.mouse_position[0], self.mouse_position[1])
            .trans(-5.0, -5.0);

        rectangle(MOUSE_COLOR, square, transform, gl);
    }
}
