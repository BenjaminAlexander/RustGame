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

pub const STEP_DURATION: TimeDuration = TimeDuration::from_millis(16);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleState {
    next_bullet_key: u32,
    player_characters: Vec<Character>,
    bullets: HashMap<u32, Bullet>
}

impl State for SimpleState {

    fn new(player_count: usize) -> Self {

        let mut new = Self{
            next_bullet_key: 0,
            player_characters: Vec::new(),
            bullets: HashMap::new(),
        };

        for i in 0..player_count {
            let character = Character::new(Vector2::new((i * 100) as f64, 0 as f64));
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

        self.bullets.retain(|key, value|{
            return !value.should_remove();
        });

        for i in 0..self.player_characters.len() {
            let input = arg.get_input(i);

            self.player_characters[i].move_character(input);

            if let Some(bullet) = self.player_characters[i].get_fired_bullet(input) {
                self.bullets.insert(self.next_bullet_key, bullet);
                self.next_bullet_key = self.next_bullet_key + 1;
            }
        }

        for bullet in self.bullets.values_mut() {
            bullet.update();
        }
    }

    pub fn draw(&self, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {
        for character in &self.player_characters {
            character.draw(args, context, gl);
        }

        for bullet in self.bullets.values() {
            bullet.draw(args, context, gl);
        }
    }
}