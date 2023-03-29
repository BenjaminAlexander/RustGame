use commons::factory::FactoryTrait;
use commons::net::TcpListenerTrait;
use crate::interface::GameFactoryTrait;

pub type TcpListener<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpListener;
pub type TcpStream<GameFactory> = <TcpListener<GameFactory> as TcpListenerTrait>::TcpStream;