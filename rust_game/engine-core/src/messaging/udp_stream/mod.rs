use std::net::SocketAddr;

use commons::real_time::net::udp::UdpSocket;
use log::warn;
use serde::{Serialize, de::DeserializeOwned};

use crate::messaging::{FragmentAssembler, Fragmenter};

pub struct UdpStreamSender {
    fragmenter: Fragmenter
}

//TODO: add flush function
impl UdpStreamSender {

    pub fn new(max_datagram_size: usize) -> Self {
        Self {
            fragmenter: Fragmenter::new(max_datagram_size)
        }
    }

    pub fn send<T: Serialize>(&mut self, udp_socket: &mut UdpSocket, destination: &SocketAddr, message: &T) -> Result<(), ()> {

        let mut fragments = match self.fragmenter.make_fragments(message) {
            Ok(fragments) => fragments,
            Err(error) => {
                warn!("An error occured while making message fragments: {:?}", error);
                return Err(())
            },
        };

        while let Some(fragment) = fragments.next() {
            match udp_socket.send_to(fragment.get_buffer(), destination) {
                Ok(bytes_sent) => {
                    if bytes_sent != fragment.get_buffer().len() {
                        warn!("Failed to send all bytes: {}/{} sent", bytes_sent, fragment.get_buffer().len());
                        return Err(())
                    }
                },
                Err(error) => {
                    warn!("IO error: {:?}", error);
                    return Err(())
                },
            }
        }

        return Ok(());
    }
}

pub struct UdpStreamReceiver {
    fragmenter_assembler: FragmentAssembler
}

impl UdpStreamReceiver {

    pub fn new(max_datagram_size: usize, max_messages: usize) -> Self {
        Self {
            fragmenter_assembler: FragmentAssembler::new(max_datagram_size, max_messages)
        }
    }

    pub fn receive<T: DeserializeOwned>(&mut self, buf: &[u8]) -> Option<T>{
        self.fragmenter_assembler.add_fragment(buf)
    }

}