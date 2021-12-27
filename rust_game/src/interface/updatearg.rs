use crate::interface::ServerUpdateArg;
use crate::interface::game::GameTrait;

#[derive(Debug)]
pub struct ClientUpdateArg<'a, 'b, 'c, Game: GameTrait> {
    server_update_arg: ServerUpdateArg<'a, 'b, Game>,
    server_input: Option<&'c Game::ServerInput>
}

impl<'a, 'b, 'c, Game: GameTrait> ClientUpdateArg<'a, 'b, 'c, Game> {

    pub fn new(server_update_arg: ServerUpdateArg<'a, 'b, Game>,
               server_input: Option<&'c Game::ServerInput>) -> Self {
        return Self{
            server_update_arg,
            server_input
        }
    }

    pub fn get_input(&self, player_index: usize) -> Option<&Game::ClientInput> {
        return self.server_update_arg.get_input(player_index);
    }

    pub fn get_current_step(&self) -> usize {
        return self.server_update_arg.get_current_step();
    }

    pub fn get_server_input(&self) -> Option<&Game::ServerInput> {
        return self.server_input;
    }
}