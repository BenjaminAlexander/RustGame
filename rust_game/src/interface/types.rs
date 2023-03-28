use commons::factory::FactoryTrait;
use commons::ip::TcpListenerTrait;
use crate::interface::GameFactoryTrait;
use crate::messaging::{ToClientMessageTCP, ToServerMessageTCP};

pub type ServerToClientTcpStream<GameFactory> = <<<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpListener as TcpListenerTrait>::TcpStream<ToServerMessageTCP, ToClientMessageTCP<<GameFactory as GameFactoryTrait>::Game>>;