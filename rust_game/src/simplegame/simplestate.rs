use crate::simplegame::{Vector2, SimpleInputEvent, SimpleInput};
use crate::interface::{State, NextStateArg, StateUpdate};
use serde::{Deserialize, Serialize};
use crate::simplegame::character::Character;
use opengl_graphics::GlGraphics;
use graphics::Context;
use piston::RenderArgs;
use crate::gametime::TimeDuration;
use crate::simplegame::bullet::Bullet;
use std::collections::HashMap;

pub const STEP_DURATION: TimeDuration = TimeDuration::from_millis(500);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleState {
    player_characters: Vec<Character>,
    bullets: Vec<Bullet>
}

impl State for SimpleState {

    fn new(player_count: usize) -> Self {

        let mut new = Self{
            player_characters: Vec::new(),
            bullets: Vec::new(),
        };

        for i in 0..player_count {
            let character = Character::new(
                i,
                Vector2::new((i * 100) as f64, 0 as f64)
            );

            new.player_characters.push(character);
        }

        return new;
    }
}

impl StateUpdate<SimpleState, SimpleInput> for SimpleState {

    fn get_next_state(state: &SimpleState, arg: &NextStateArg<SimpleInput>) -> SimpleState {
        let mut new = state.clone();
        new.update(arg);
        return new;
    }

}

impl SimpleState {

    fn update(&mut self, arg: &NextStateArg<SimpleInput>) {

        let duration_of_start_to_current = STEP_DURATION * arg.get_current_step() as i64;

        let mut i = 0;
        while i < self.bullets.len() {
            if self.bullets[i].should_remove(duration_of_start_to_current) {
                self.bullets.remove(i);
            } else {
                i = i + 1;
            }
        }

        for i in 0..self.player_characters.len() {
            let input = arg.get_input(i);

            self.player_characters[i].move_character(&arg);

            if let Some(bullet) = self.player_characters[i].get_fired_bullet(&arg) {
                self.bullets.push(bullet);
            }
        }
    }

    pub fn draw(&self, duration_since_game_start: TimeDuration, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {
        for character in &self.player_characters {
            character.draw(args, context, gl);
        }

        for bullet in &self.bullets {
            bullet.draw(duration_since_game_start, args, context, gl);
        }
    }
}