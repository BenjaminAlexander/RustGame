use crate::interface::{InputEvent, Input};

pub trait InputEventHandler<InputType: Input, InputEventType: InputEvent>: Send + 'static {

    fn new() -> Self;

    fn handle_event(&mut self, input_event: InputEventType);

    fn get_input(&mut self) -> InputType;

}