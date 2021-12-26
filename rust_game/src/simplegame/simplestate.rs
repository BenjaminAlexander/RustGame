use crate::simplegame::{Vector2, SimplServerInputEvent};
use crate::interface::{State, ClientUpdateArg, InterpolationArg, InterpolationResult, ServerUpdateArg, Game};
use serde::{Deserialize, Serialize};
use crate::simplegame::character::Character;
use opengl_graphics::GlGraphics;
use graphics::Context;
use piston::RenderArgs;
use crate::gametime::TimeDuration;
use crate::simplegame::bullet::Bullet;
use crate::messaging::InitialInformation;
use crate::simplegame::simpleserverinput::SimpleServerInput;
use crate::SimpleGameImpl;

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

impl SimpleState {

    pub fn get_server_input(state: &SimpleState, arg: &ServerUpdateArg<SimpleGameImpl>) -> SimpleServerInput {
        let mut server_input = SimpleServerInput::new();

        for character in &state.player_characters {
            for bullet in &state.bullets {
                if character.is_hit(bullet, arg.get_current_duration_since_start()) {
                    server_input.add_event(SimplServerInputEvent::CharacterHit{
                        index: character.get_player_index()
                    });
                }
            }
        }

        return server_input;
    }

    pub fn get_next_state(state: &SimpleState, arg: &ClientUpdateArg<SimpleGameImpl>) -> SimpleState {
        let mut new = state.clone();
        new.update(arg);
        return new;
    }

    fn update(&mut self, arg: &ClientUpdateArg<SimpleGameImpl>) {

        if let Some(server_input) = arg.get_server_input() {
            server_input.apply_to_state(self);
        }

        let duration_of_start_to_current = SimpleGameImpl::STEP_PERIOD * arg.get_current_step() as i64;

        let mut i = 0;
        while i < self.bullets.len() {
            if self.bullets[i].should_remove(duration_of_start_to_current) {
                self.bullets.remove(i);
            } else {
                i = i + 1;
            }
        }

        for i in 0..self.player_characters.len() {
            if let Some(bullet) = self.player_characters[i].get_fired_bullet(&arg) {
                self.bullets.push(bullet);
            }

            self.player_characters[i].move_character(&arg);
        }
    }

    pub fn interpolate(_initial_information: &InitialInformation<SimpleGameImpl>, first: &Self, second: &Self, arg: &InterpolationArg) -> Self {

        let mut second_clone = second.clone();

        for i in 0..second_clone.player_characters.len() {
            if let Some(first_character) = first.player_characters.get(i) {
                let new_position = first_character.get_position().lerp(second_clone.player_characters[i].get_position(), arg.get_weight());
                second_clone.player_characters[i].set_position(new_position);
            }
        }

        return second_clone;
    }

    pub fn draw(&self, duration_since_game_start: TimeDuration, args: &RenderArgs, context: Context, gl: &mut GlGraphics) {
        for character in &self.player_characters {
            character.draw(args, context, gl);
        }

        for bullet in &self.bullets {
            bullet.draw(duration_since_game_start, args, context, gl);
        }
    }

    pub fn hit_character(&mut self, index: usize) {
        self.player_characters[index].reduce_health();
    }
}

impl InterpolationResult for SimpleState {

}