use commons::geometry::twod::Vector2;
use piston::{ButtonState, ButtonArgs, Button, Key, Motion, MouseButton};
use piston::input::Input as PistonInput;
use crate::simplegame::{SimpleInput, SimpleInputEvent};

pub struct SimpleInputEventHandler {
    aim_point: Vector2,
    d_state: ButtonState,
    a_state: ButtonState,
    s_state: ButtonState,
    w_state: ButtonState,
    left_mouse_state: ButtonState,
    should_fire: bool
}

impl SimpleInputEventHandler {

    pub fn new() -> Self {
        Self{
            aim_point: Vector2::new(0 as f64, 0 as f64),
            d_state: ButtonState::Release,
            a_state: ButtonState::Release,
            s_state: ButtonState::Release,
            w_state: ButtonState::Release,
            left_mouse_state: ButtonState::Release,
            should_fire: false
        }
    }

    pub fn handle_event(&mut self, input_event: SimpleInputEvent) {
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

    pub fn get_input(&mut self) -> SimpleInput {

        let x = match (self.d_state, self.a_state) {
            (ButtonState::Press, ButtonState::Press) => 0,
            (ButtonState::Release, ButtonState::Press) => -1,
            (ButtonState::Press, ButtonState::Release) => 1,
            (ButtonState::Release, ButtonState::Release) => 0,
        } as f64;

        let y = match (self.s_state, self.w_state) {
            (ButtonState::Press, ButtonState::Press) => 0,
            (ButtonState::Release, ButtonState::Press) => -1,
            (ButtonState::Press, ButtonState::Release) => 1,
            (ButtonState::Release, ButtonState::Release) => 0,
        } as f64;

        let velocity = Vector2::new(x, y).normalize();

        let input = SimpleInput::new(
            self.aim_point,
            velocity,
            self.should_fire
        );

        self.should_fire = false;

        return input;
    }

    fn accumulate_move(&mut self, move_event: &Motion) {
        match move_event {
            Motion::MouseCursor(position) => {
                self.aim_point = Vector2::new(position[0], position[1]);
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
            Button::Mouse(mouse_button) => {
                match mouse_button {
                    MouseButton::Left => {
                        if self.left_mouse_state == ButtonState::Release &&
                            button.state == ButtonState::Press {
                            self.should_fire = true;
                        }

                        self.left_mouse_state = button.state;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}