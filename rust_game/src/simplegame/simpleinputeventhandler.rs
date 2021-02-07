use piston::{ButtonState, ButtonArgs, Button, Key, Motion};
use piston::input::Input as PistonInput;
use crate::interface::InputEventHandler;
use crate::simplegame::{SimpleInput, SimpleInputEvent, Vector2};
use log::info;
use num::integer::Roots;

pub struct SimpleInputEventHandler {
    vector_option: Option<Vector2>,
    d_state: ButtonState,
    a_state: ButtonState,
    s_state: ButtonState,
    w_state: ButtonState
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
                match key {
                    Key::D => self.d_state = button.state,
                    Key::A => self.a_state = button.state,
                    Key::S => self.s_state = button.state,
                    Key::W => self.w_state = button.state,
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
            d_state: ButtonState::Release,
            a_state: ButtonState::Release,
            s_state: ButtonState::Release,
            w_state: ButtonState::Release
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

        let mut x = match (self.d_state, self.a_state) {
            (ButtonState::Press, ButtonState::Press) => 0,
            (ButtonState::Release, ButtonState::Press) => -1,
            (ButtonState::Press, ButtonState::Release) => 1,
            (ButtonState::Release, ButtonState::Release) => 0,
        } as f64;

        let mut y = match (self.s_state, self.w_state) {
            (ButtonState::Press, ButtonState::Press) => 0,
            (ButtonState::Release, ButtonState::Press) => -1,
            (ButtonState::Press, ButtonState::Release) => 1,
            (ButtonState::Release, ButtonState::Release) => 0,
        } as f64;

        let h = (x.powf(2 as f64) + y.powf(2 as f64)).sqrt();

        if h != 0 as f64 {
            let cos = x / h;
            let sin = y / h;

            x = cos;
            y = sin;
        }

        let input = SimpleInput::new(self.vector_option, Vector2::new(x as f64, y as f64));

        self.vector_option = None;

        return input;
    }
}