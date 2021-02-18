use crate::simplegame::{Vector2, SimpleInputEvent, SimpleInput};
use crate::interface::{State, NextStateArg};
use serde::{Deserialize, Serialize};
use crate::simplegame::character::Character;
use opengl_graphics::GlGraphics;
use graphics::Context;
use piston::RenderArgs;
use crate::gametime::TimeDuration;
use crate::simplegame::bullet::Bullet;

//TODO: 8 mills causes some strange input loss
//TODO: its connected to n-1,n,n+1 in core when timer messages come in
pub const STEP_DURATION: TimeDuration = TimeDuration::from_millis(50);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleState {
    player_characters: Vec<Character>,
    bullets: Vec<Bullet>
}

impl State<SimpleInput> for SimpleState {

    fn new(player_count: usize) -> Self {

        let mut new = Self{
            player_characters: Vec::new(),
            bullets: Vec::new(),
        };

        for i in 0..player_count {
            let character = Character::new(Vector2::new((i * 100) as f64, 0 as f64));
            new.player_characters.push(character);
        }

        return new;
    }

    fn get_next_state(&self, arg: &NextStateArg<SimpleInput>) -> Self {
        let mut new = self.clone();
        new.update(arg);
        return new;
    }
}

impl SimpleState {

    fn update(&mut self, arg: &NextStateArg<SimpleInput>) {

        let mut i = 0;
        while i < self.bullets.len() {
            if self.bullets[i].should_remove() {
                self.bullets.remove(i);
            } else {
                i = i + 1;
            }
        }

        for i in 0..self.player_characters.len() {
            let input = arg.get_input(i);

            self.player_characters[i].move_character(input);

            if let Some(bullet) = self.player_characters[i].get_fired_bullet(input) {
                self.bullets.push(bullet);
            }
        }

        for bullet in &mut self.bullets {
            bullet.update();
        }
    }

    pub fn draw(&self, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {
        for character in &self.player_characters {
            character.draw(args, context, gl);
        }

        for bullet in &self.bullets {
            bullet.draw(args, context, gl);
        }
    }
}