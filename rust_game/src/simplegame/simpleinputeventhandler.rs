use piston::{ButtonState, ButtonArgs, Button, Key, Motion};
use piston::input::Input as PistonInput;
use crate::interface::InputEventHandler;
use crate::simplegame::{SimpleInput, SimpleInputEvent, Vector2};
use log::info;

pub struct SimpleInputEventHandler {
    vector_option: Option<Vector2>,
    d_state: ButtonState
}

impl SimpleInputEventHandler {

    fn accumulate_move(&mut self, move_event: &Motion) {
        match move_event {
            Motion::MouseCursor(position) => {
                self.vector_option = Some(Vector2::new(position[0], position[1]));
            }
            _ => {}
        }
    }

    fn accumulate_button(&mut self, button: &ButtonArgs) {


        match button.button {
            Button::Keyboard(key) => {

                info!("{:?}", button);

                match key {
                    Key::D => self.d_state = button.state,
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl InputEventHandler<SimpleInput, SimpleInputEvent> for SimpleInputEventHandler {
    fn new() -> Self {
        Self{
            vector_option: None,
            d_state: ButtonState::Release
        }
    }

    fn handle_event(&mut self, input_event: SimpleInputEvent) {
        match input_event.get_piston_input() {
            PistonInput::Button(arg) => {
                self.accumulate_button(arg);
            }
            PistonInput::Move(move_event) => {
                self.accumulate_move(move_event);
            }
            PistonInput::Text(_) => {}
            PistonInput::Resize(_) => {}
            PistonInput::Focus(_) => {}
            PistonInput::Cursor(_) => {}
            PistonInput::FileDrag(_) => {}
            PistonInput::Close(_) => {}
        }
    }

    fn get_input(&mut self) -> SimpleInput {

        let velocity = if self.d_state == ButtonState::Press {
            Vector2::new(1 as f64, 0 as f64)
        } else {
            Vector2::new(0 as f64, 0 as f64)
        };

        let input = SimpleInput::new(self.vector_option, velocity);

        self.vector_option = None;

        return input;
    }
}