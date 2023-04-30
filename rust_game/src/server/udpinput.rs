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

//TODO: timeout fragments or fragment assemblers

pub struct UdpInput<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    socket: UdpSocket,
    remote_peers: Vec<Option<RemoteUdpPeer>>,
    client_addresses: Vec<Option<ClientAddress>>,
    client_ip_set: HashSet<IpAddr>,
    fragment_assemblers: HashMap<SocketAddr, FragmentAssembler>,
    core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {

    pub fn new(
        factory: GameFactory::Factory,
        socket: &UdpSocket,
        core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>) -> io::Result<Self> {

        return Ok(Self {
            factory,
            socket: socket.try_clone()?,
            remote_peers: Vec::new(),
            client_addresses: Vec::new(),
            client_ip_set: HashSet::new(),
            //TODO: make this more configurable
            fragment_assemblers: HashMap::new(),
            core_sender
        });
    }

    fn channel_empty_after_listen(&mut self, mut buf: [u8; MAX_UDP_DATAGRAM_SIZE], number_of_bytes: usize, source: SocketAddr) {
        self.core_sender.send_event(ServerCoreEvent::UdpPacket(source, number_of_bytes, buf)).unwrap();
    }
}

impl<Game: GameFactoryTrait> UdpReadHandlerTrait for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buff: &[u8]) -> ControlFlow<()> {
        todo!()
    }
}

impl<Game: GameFactoryTrait> ListenerTrait for UdpInput<Game> {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = ([u8; MAX_UDP_DATAGRAM_SIZE], usize, SocketAddr);

    fn listen(self) -> ListenResult<Self> {
        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

        let recv_result = self.socket.recv_from(&mut buf);

        match recv_result {
            Ok((number_of_bytes, source)) => {
                return Continue(Listened(self, (buf, number_of_bytes, source)));
            }
            Err(e) => {
                warn!("Error: {:?}", e);
                return Continue(DidNotListen(self));
            }
        }
    }

    fn on_channel_event(mut self, event: ChannelEvent<Self>) -> ListenerEventResult<Self> {
        match event {
            ChannelEvent::ChannelEmptyAfterListen(_, (buf, number_of_bytes, source)) => {
                self.channel_empty_after_listen(buf, number_of_bytes, source);
                return Continue(self);
            }
            ChannelEvent::ReceivedEvent(_, ()) => {
                return Continue(self);
            }
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}