use log::{info, warn};
use crate::messaging::{ToServerMessageUDP, FragmentAssembler, MessageFragment};
use crate::interface::GameFactoryTrait;
use std::net::{UdpSocket, SocketAddr, IpAddr};
use std::io;
use crate::server::remoteudppeer::RemoteUdpPeer;
use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use commons::net::{MAX_UDP_DATAGRAM_SIZE, UdpReadHandlerTrait};
use crate::server::clientaddress::ClientAddress;
use crate::server::servercore::ServerCoreEvent;
use commons::threading::channel::{ReceiveMetaData, SenderTrait};
use commons::threading::eventhandling::{Sender, EventSenderTrait};
use commons::threading::listener::{ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use commons::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};

pub struct UdpInput<GameFactory: GameFactoryTrait> {
    core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {

    pub fn new(core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self {
            core_sender
        };
    }

}

impl<Game: GameFactoryTrait> UdpReadHandlerTrait for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let mut buf_to_send = [0; MAX_UDP_DATAGRAM_SIZE];
        buf_to_send[..buf.len()].copy_from_slice(buf);
        self.core_sender.send_event(ServerCoreEvent::UdpPacket(peer_addr, buf.len(), buf_to_send)).unwrap();
        return Continue(());
    }
}