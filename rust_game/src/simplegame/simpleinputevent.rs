use piston::input::Input as PistonInput;

pub struct SimpleInputEvent(PistonInput);

impl SimpleInputEvent {

    pub fn new(piston_input: PistonInput) -> Self {
        return SimpleInputEvent(piston_input);
    }

    pub fn get_piston_input(&self) -> &PistonInput {
        return &self.0
    }
}