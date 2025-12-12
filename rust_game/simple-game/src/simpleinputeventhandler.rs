use crate::simpleinput::SimpleInput;
use crate::simpleinputevent::SimpleInputEvent;
use commons::geometry::twod::Vector2;
use engine_core::AggregateInput;
use piston::input::Input as PistonInput;
use piston::{
    Button,
    ButtonArgs,
    ButtonState,
    Key,
    Motion,
    MouseButton,
};

pub struct SimpleInputEventHandler {
    aim_point: Vector2,
    d: MoveButtonTracker,
    a: MoveButtonTracker,
    s: MoveButtonTracker,
    w: MoveButtonTracker,
    left_mouse_state: ButtonState,
    should_fire: bool,
}

impl AggregateInput for SimpleInputEventHandler {
    type ClientInputEvent = SimpleInputEvent;

    type ClientInput = SimpleInput;

    fn new() -> Self {
        Self {
            aim_point: Vector2::new(0 as f64, 0 as f64),
            d: MoveButtonTracker::new(),
            a: MoveButtonTracker::new(),
            s: MoveButtonTracker::new(),
            w: MoveButtonTracker::new(),
            left_mouse_state: ButtonState::Release,
            should_fire: false,
        }
    }

    fn handle_input_event(&mut self, input_event: Self::ClientInputEvent) {
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

    fn get_input(&mut self) -> Self::ClientInput {
        let x = match (self.d.take_was_down(), self.a.take_was_down()) {
            (true, true) => 0,
            (false, true) => -1,
            (true, false) => 1,
            (false, false) => 0,
        } as f64;

        let y = match (self.s.take_was_down(), self.w.take_was_down()) {
            (true, true) => 0,
            (false, true) => -1,
            (true, false) => 1,
            (false, false) => 0,
        } as f64;

        let velocity = Vector2::new(x, y).normalize();

        let input = SimpleInput::new(self.aim_point, velocity, self.should_fire);

        self.should_fire = false;

        return input;
    }
}

impl SimpleInputEventHandler {

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
            Button::Keyboard(key) => match key {
                Key::D => self.d.set_state(button.state),
                Key::A => self.a.set_state(button.state),
                Key::S => self.s.set_state(button.state),
                Key::W => self.w.set_state(button.state),
                _ => {}
            },
            Button::Mouse(mouse_button) => match mouse_button {
                MouseButton::Left => {
                    if self.left_mouse_state == ButtonState::Release
                        && button.state == ButtonState::Press
                    {
                        self.should_fire = true;
                    }

                    self.left_mouse_state = button.state;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

struct MoveButtonTracker {
    last_state: ButtonState,
    was_down: bool
}

impl MoveButtonTracker {
    fn new() -> Self {
        Self {
            last_state: ButtonState::Release,
            was_down: false
        }
    }

    fn set_state(&mut self, new_state: ButtonState) {
        self.last_state = new_state;
        if self.last_state == ButtonState::Press {
            self.was_down = true;
        }
    }

    fn take_was_down(&mut self) -> bool {
        let value = self.was_down;
        self.was_down = false;
        return value || self.last_state == ButtonState::Press;
    }
}
