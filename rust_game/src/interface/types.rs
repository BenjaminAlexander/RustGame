use commons::factory::FactoryTrait;
use commons::net::TcpListenerTrait;
use crate::interface::GameFactoryTrait;
use crate::messaging::{ToClientMessageTCP, ToServerMessageTCP};

pub type ServerTcpListener<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpListener<ToServerMessageTCP, ToClientMessageTCP<<GameFactory as GameFactoryTrait>::Game>>;
pub type ServerToClientTcpStream<GameFactory> = <ServerTcpListener<GameFactory> as TcpListenerTrait>::TcpStream;