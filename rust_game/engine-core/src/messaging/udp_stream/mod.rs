use std::net::SocketAddr;

use commons::real_time::net::udp::UdpSocket;
use log::warn;
use serde::Serialize;

use crate::messaging::Fragmenter;

pub struct UdpStream {
    fragmenter: Fragmenter
}

impl UdpStream {

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
            match udp_socket.send_to(&fragment.get_whole_buf(), destination) {
                Ok(bytes_sent) => {
                    if bytes_sent != fragment.get_whole_buf().len() {
                        warn!("Failed to send all bytes: {}/{} sent", bytes_sent, fragment.get_whole_buf().len());
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