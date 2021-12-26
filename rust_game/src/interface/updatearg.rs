use crate::interface::ServerUpdateArg;
use crate::interface::game::GameTrait;

#[derive(Debug)]
pub struct ClientUpdateArg<'a, 'b, 'c, GameType: GameTrait> {
    server_update_arg: ServerUpdateArg<'a, 'b, GameType>,
    server_input: Option<&'c GameType::ServerInputType>
}

impl<'a, 'b, 'c, GameType: GameTrait> ClientUpdateArg<'a, 'b, 'c, GameType> {

    pub fn new(server_update_arg: ServerUpdateArg<'a, 'b, GameType>,
               server_input: Option<&'c GameType::ServerInputType>) -> Self {
        return Self{
            server_update_arg,
            server_input
        }
    }

    pub fn get_input(&self, player_index: usize) -> Option<&GameType::InputType> {
        return self.server_update_arg.get_input(player_index);
    }

    pub fn get_current_step(&self) -> usize {
        return self.server_update_arg.get_current_step();
    }

    pub fn get_server_input(&self) -> Option<&GameType::ServerInputType> {
        return self.server_input;
    }
}