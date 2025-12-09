use crate::bullet::Bullet;
use crate::character::Character;
use crate::simpleserverinput::SimplServerInputEvent;
use crate::simpleserverinput::SimpleServerInput;
use crate::SimpleGameImpl;
use commons::geometry::twod::Vector2;
use commons::time::TimeDuration;
use engine_core::{
    GameTrait,
    InitialInformation,
    InterpolationArg,
    UpdateArg,
};
use graphics::Context;
use opengl_graphics::GlGraphics;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleState {
    player_characters: Vec<Character>,
    bullets: Vec<Bullet>,
}

impl SimpleState {
    pub fn new(player_count: usize) -> Self {
        let mut new = Self {
            player_characters: Vec::new(),
            bullets: Vec::new(),
        };

        for i in 0..player_count {
            let character = Character::new(i, Vector2::new((i * 100) as f64, 0 as f64));

            new.player_characters.push(character);
        }

        return new;
    }

    pub fn get_server_input(arg: &UpdateArg<SimpleGameImpl>) -> SimpleServerInput {
        let mut server_input = SimpleServerInput::new();

        for character in &arg.get_state().player_characters {
            for bullet in &arg.get_state().bullets {
                if character.is_hit(bullet, arg.get_current_duration_since_start()) {
                    server_input.add_event(SimplServerInputEvent::CharacterHit {
                        index: character.get_player_index(),
                    });
                }
            }
        }

        return server_input;
    }

    pub fn get_next_state(arg: &UpdateArg<SimpleGameImpl>) -> SimpleState {
        let mut new = arg.get_state().clone();
        new.update(arg);
        return new;
    }

    fn update(&mut self, arg: &UpdateArg<SimpleGameImpl>) {
        //TODO: This should probably check the authoritativeness of the input to do server-only logic
        let server_input = Self::get_server_input(arg);
        server_input.apply_to_state(self);

        //TODO: refactor this time calculation
        let duration_of_start_to_current =
            SimpleGameImpl::STEP_PERIOD.mul_f64(arg.get_current_step().usize() as f64);

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

    pub fn interpolate(
        _initial_information: &InitialInformation<SimpleGameImpl>,
        first: &Self,
        second: &Self,
        arg: &InterpolationArg,
    ) -> Self {
        let mut second_clone = second.clone();

        for i in 0..second_clone.player_characters.len() {
            if let Some(first_character) = first.player_characters.get(i) {
                let new_position = first_character.get_position().lerp(
                    second_clone.player_characters[i].get_position(),
                    arg.get_weight(),
                );
                second_clone.player_characters[i].set_position(new_position);
            }
        }

        return second_clone;
    }

    pub fn draw(
        &self,
        initial_information: &InitialInformation<SimpleGameImpl>,
        duration_since_game_start: TimeDuration,
        context: Context,
        gl: &mut GlGraphics,
    ) {
        for character in &self.player_characters {
            character.draw(context, gl, initial_information.get_player_index());
        }

        for bullet in &self.bullets {
            bullet.draw(duration_since_game_start, context, gl);
        }
    }

    pub fn hit_character(&mut self, index: usize) {
        self.player_characters[index].reduce_health();
    }
}
