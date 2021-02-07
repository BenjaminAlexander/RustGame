use crate::simplegame::{Vector2, SimpleInputEvent, SimpleInput};
use crate::interface::{State, NextStateArg};
use serde::{Deserialize, Serialize};
use crate::simplegame::character::Character;
use opengl_graphics::GlGraphics;
use graphics::Context;
use piston::RenderArgs;
use crate::gametime::TimeDuration;

pub const STEP_DURATION: TimeDuration = TimeDuration::from_millis(16);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleState {
    pub player_characters: Vec<Character>
}

impl State<SimpleInput> for SimpleState {

    fn new(player_count: usize) -> Self {

        let mut new = Self{player_characters: Vec::new()};

        for i in 0..player_count {
            let character = Character::new(Vector2::new((i * 100) as f64, 0 as f64));
            new.player_characters.push(character);
        }

        return new;
    }

    fn get_next_state(&self, arg: &NextStateArg<SimpleInput>) -> Self {
        let mut new = self.clone();

        for i in 0..new.player_characters.len() {
            if let Some(input) = arg.get_input(i) {
                if let Some(vector) = input.get_vector_option() {
                    new.player_characters[i].set_position(vector);
                }
            }
        }

        for i in 0..new.player_characters.len() {
            new.player_characters[i].update(arg.get_input(i));
        }

        return new;
    }
}

impl SimpleState {

    pub fn draw(&self, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {

        for character in &self.player_characters {
            character.draw(args, context, gl);
        }
    }
}