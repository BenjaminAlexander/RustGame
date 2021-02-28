use crate::interface::ServerInput;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleServerInput {

}

impl SimpleServerInput {
    pub fn new() -> Self {
        return Self{};
    }
}

impl ServerInput for SimpleServerInput {

}