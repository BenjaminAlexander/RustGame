use crate::logging::unwrap_or_log_panic;
use crate::net::TcpWriterTrait;
use log::{info, warn};
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;
use rmp_serde::Deserializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::{vec_deque, VecDeque};
use std::fmt::Debug;
use std::io::{
    self, BufRead, BufReader, Cursor, Error, Read, Write
};
use std::net::{
    SocketAddr,
    TcpStream,
};
use std::ops::ControlFlow;

#[derive(Debug)]
pub struct RealTcpStream {
    tcp_stream: TcpStream,
    remote_peer_socket_addr: SocketAddr,
}

impl RealTcpStream {
    pub fn new(tcp_stream: TcpStream, remote_peer_socket_addr: SocketAddr) -> Self {
        return Self {
            tcp_stream,
            remote_peer_socket_addr,
        };
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.remote_peer_socket_addr;
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        return self.tcp_stream.local_addr();
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        return Ok(Self {
            tcp_stream: self.tcp_stream.try_clone()?,
            remote_peer_socket_addr: self.remote_peer_socket_addr.clone(),
        });
    }

    //TODO: maybe remove?
    pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> Result<T, ControlFlow<()>> {

        //let mut deserializer: Deserializer<rmp_serde::decode::ReadReader<&TcpStream>> = Deserializer::new(&self.tcp_stream);
        let x = Deserialize::deserialize(&mut Deserializer::new(&self.tcp_stream));

        return match x {
            Ok(value) => Ok(value),
            Err(DecodeError::InvalidMarkerRead(ref error)) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => Err(ControlFlow::Continue(())),
            Err(error) => {
                warn!("Error on TCP read: {:?}", error);
                Err(ControlFlow::Break(()))
            },
        };
    }

    pub fn to_deserializer(self) -> TcpDeserializer {
        return TcpDeserializer {
            //deserializer: Deserializer::new(self.tcp_stream),
            buf_reader: BufReader::new(self.tcp_stream),
            buf: VecDeque::new(),
            fill_len: 0,
            read_len: 0,
            remote_peer_socket_addr: self.remote_peer_socket_addr,
        }
    }
    
}

impl TcpWriterTrait for RealTcpStream {
    fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError> {
        return rmp_serde::encode::write(&mut self.tcp_stream, &write);
    }

    fn flush(&mut self) -> Result<(), Error> {
        return self.tcp_stream.flush();
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return RealTcpStream::get_peer_addr(self);
    }
}

pub struct TcpDeserializer {
    //deserializer: Deserializer<rmp_serde::decode::ReadReader<TcpStream>>,
    buf_reader: BufReader<TcpStream>,
    buf: VecDeque<u8>,
    fill_len: usize,
    read_len: usize,
    remote_peer_socket_addr: SocketAddr,
}

impl TcpDeserializer {

    pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> Result<T, ControlFlow<()>> {
/*
        let c = Cursor::new(self.buf);

        VecDeque

        c.read(buf)

        warn!("BEGIN");

        //let i_buf = self.buf_reader.fill_buf().unwrap();
        warn!("i_buf: {:?}", self.buf_reader.fill_buf());
*/
/*
        let mut buf: [u8; 1] = [0];
        self.buf_reader.read(&mut buf);
        warn!("buf: {:?}", buf);

        self.buf_reader.read(&mut buf);
        warn!("buf: {:?}", buf);




        self.buf_reader.consume(1);
        let i_buf = self.buf_reader.fill_buf().unwrap();
        warn!("i_buf: {:?}", i_buf);

        self.buf_reader.read(&mut buf);
        warn!("buf: {:?}", buf);
*/

        //return Err(ControlFlow::Continue(()));

        //let result = Deserialize::deserialize(&mut self.deserializer);
        let result = rmp_serde::decode::from_read(self);

        return match result {
            Ok(value) => Ok(value),
            Err(DecodeError::InvalidMarkerRead(ref error)) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
                    //TODO: reset reader
                    
                    Err(ControlFlow::Continue(()))
            },
            Err(error) => {
                warn!("Error on TCP read: {:?}", error);
                Err(ControlFlow::Break(()))
            },
        };
    }

}

impl Read for TcpDeserializer {

    fn read(&mut self, read_buf: &mut [u8]) -> io::Result<usize> {

        warn!("Buf: {:?}", self.buf);

        let unread_bytes_in_buf = self.fill_len - self.read_len;
        let bytes_needed_from_tcp_stream = read_buf.len() - unread_bytes_in_buf;
        
        if bytes_needed_from_tcp_stream > self.buf.len() - self.fill_len {
            self.buf.resize(self.fill_len + bytes_needed_from_tcp_stream, 0);
        }

        if bytes_needed_from_tcp_stream > 0 {

            //Need to read bytes from the TcpStream

            let slice = self.buf.make_contiguous();

            let end = bytes_needed_from_tcp_stream + self.fill_len;
            let slice_to_read_into = &mut slice[self.fill_len..end];

            //TODO: remove
            debug_assert!(read_buf.len() - slice_to_read_into.len() - unread_bytes_in_buf == 0);

            let result = self.buf_reader.read(slice_to_read_into);

            match result {
                Ok(read_len) => {
                    warn!("Buf after read: {:?}", self.buf);
                    self.fill_len += read_len;              
                },
                Err(error) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
                    return Err(Error::from(io::ErrorKind::TimedOut));
                }
                Err(_) => return result,
            }

        }

        //Now, the bytes have already been buffered

        //TODO: start here:
        let bytes_available = self.buf.len() - self.read_len;
        let len_to_read = min(bytes_available, read_buf.len());

        //read_buf.clone_from_slice(src)



    /*     


         */   

        let result = self.buf_reader.read(read_buf);
        warn!("READ: {:?}", result);
        return result;
    }
}