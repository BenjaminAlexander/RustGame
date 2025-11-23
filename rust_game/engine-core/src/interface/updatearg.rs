use crate::gamemanager::Input;
use crate::gametime::FrameIndex;
use crate::interface::game::GameTrait;
use crate::interface::ServerUpdateArg;

#[derive(Debug)]
pub struct ClientUpdateArg<'a, Game: GameTrait> {
    server_update_arg: ServerUpdateArg<'a, Game>,
    server_input: Option<&'a Game::ServerInput>,
}

impl<'a, Game: GameTrait> ClientUpdateArg<'a, Game> {
    pub fn new(
        server_update_arg: ServerUpdateArg<'a, Game>,
        server_input: Option<&'a Game::ServerInput>,
    ) -> Self {
        return Self {
            server_update_arg,
            server_input,
        };
    }

    pub fn get_input(&self, player_index: usize) -> &Input<Game::ClientInput> {
        return self.server_update_arg.get_input(player_index);
    }

    pub fn get_current_step(&self) -> FrameIndex {
        return self.server_update_arg.get_current_step();
    }

    pub fn get_state(&self) -> &Game::State {
        return self.server_update_arg.get_state();
    }

    pub fn get_server_input(&self) -> Option<&Game::ServerInput> {
        return self.server_input;
    }
}
