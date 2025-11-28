pub use self::serverconfig::ServerConfig;
pub use self::servercore::{
    //TODO: make private
    ServerCoreEventHandler,
    ServerCoreEvent,
    ServerCore,
    ServerCoreBuilder,
};
pub use crate::server::tcpconnectionhandler::TcpConnectionHandler;

mod clientaddress;
mod remoteudppeer;
mod serverconfig;
mod servercore;
mod servergametimerobserver;
mod servermanagerobserver;
mod tcpconnectionhandler;
mod tcpinput;
mod tcpoutput;
mod udphandler;
mod udpinput;
mod udpoutput;
