use crate::simplegame::{Vector2, SimpleInputEvent};
use crate::interface::Input;
use piston::{Motion, ButtonArgs, Button, Key, ButtonState};
use piston::input::Input as PistonInput;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleInput {
    vector_option: Option<Vector2>,
    d_option: Option<ButtonState>
}

impl SimpleInput {

    pub fn get_vector_option(&self) -> Option<Vector2> {
        return self.vector_option;
    }

    pub fn get_d_option(&self) -> Option<ButtonState> {
        return self.d_option;
    }

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
                    Key::D => self.d_option = Some(button.state),
                    _ => {}
                }
            }
            _ => {}
        }

        match button {
            ButtonArgs { state, button, .. } => {

            }
        }
    }
}

impl Input<SimpleInputEvent> for SimpleInput {

    fn new() -> Self {
        return Self{
            vector_option: None,
            d_option: None
        };
    }

    fn accumulate(&mut self, input_event: SimpleInputEvent) {

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
}