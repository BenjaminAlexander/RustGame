use {
    crate::{
        simplegameimpl::SimpleGameImpl,
        simpleinput::SimpleInput,
        simpleinputevent::SimpleInputEvent,
    },
    commons::geometry::twod::Vector2,
    engine_core::AggregateInput,
    piston::{
        Button,
        ButtonArgs,
        ButtonState,
        Key,
        Motion,
        MouseButton,
        input::Input as PistonInput,
    },
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
    type Game = SimpleGameImpl;

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

    fn aggregate_input_event(
        &mut self,
        fractional_frame_duration: f64,
        input_event: SimpleInputEvent,
    ) -> Option<SimpleInput> {
        match input_event.get_piston_input() {
            PistonInput::Button(arg) => {
                self.accumulate_button(fractional_frame_duration, arg);
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

        Some(self.get_current_aggregate())
    }

    fn start_next_frame(&mut self) -> SimpleInput {
        self.should_fire = false;
        self.a.reset_for_new_frame();
        self.d.reset_for_new_frame();
        self.s.reset_for_new_frame();
        self.w.reset_for_new_frame();

        self.get_current_aggregate()
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

    fn accumulate_button(&mut self, fractional_frame_duration: f64, button: &ButtonArgs) {
        match button.button {
            Button::Keyboard(key) => match key {
                Key::D => self.d.set_state(fractional_frame_duration, button.state),
                Key::A => self.a.set_state(fractional_frame_duration, button.state),
                Key::S => self.s.set_state(fractional_frame_duration, button.state),
                Key::W => self.w.set_state(fractional_frame_duration, button.state),
                _ => {}
            },
            Button::Mouse(mouse_button) => match mouse_button {
                MouseButton::Left => {
                    if self.left_mouse_state == ButtonState::Release
                        && button.state == ButtonState::Press
                    {
                        //TODO: should probably capture the bullet direction when the fire happens
                        self.should_fire = true;
                    }

                    self.left_mouse_state = button.state;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn get_current_aggregate(&self) -> SimpleInput {
        let x = self.d.get_total_down_time() - self.a.get_total_down_time();
        let y = self.s.get_total_down_time() - self.w.get_total_down_time();

        let velocity = Vector2::new(x, y);

        let input = SimpleInput::new(self.aim_point, velocity, self.should_fire);

        return input;
    }
}

struct MoveButtonTracker {
    state: ButtonState,
    since: f64,
    total_down_time: f64,
}

impl MoveButtonTracker {
    fn new() -> Self {
        Self {
            state: ButtonState::Release,
            since: 0.0,
            total_down_time: 0.0,
        }
    }

    fn set_state(&mut self, fractional_frame_duration: f64, new_state: ButtonState) {
        if self.state == ButtonState::Press && new_state == ButtonState::Release {
            let this_duration = fractional_frame_duration - self.since;
            self.total_down_time += this_duration;
        }

        if self.state != new_state {
            self.state = new_state;
            self.since = fractional_frame_duration;
        }
    }

    fn get_total_down_time(&self) -> f64 {
        let mut total = self.total_down_time;

        if self.state == ButtonState::Press {
            total += 1.0 - self.since;
        }

        total
    }

    fn reset_for_new_frame(&mut self) {
        self.since = 0.0;
        self.total_down_time = 0.0;
    }
}
