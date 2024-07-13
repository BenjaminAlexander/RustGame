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
            resetable_reader: ResetableReader::new(self.tcp_stream),
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
    resetable_reader: ResetableReader<TcpStream>,
    remote_peer_socket_addr: SocketAddr,
}

impl TcpDeserializer {

    pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> Result<T, ControlFlow<()>> {

        let result = rmp_serde::decode::from_read(&mut self.resetable_reader);

        return match result {
            Ok(value) => {
                self.resetable_reader.drop_read_bytes();
                Ok(value)
            },
            Err(DecodeError::InvalidMarkerRead(ref error)) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
                    self.resetable_reader.reset_cursor();
                    Err(ControlFlow::Continue(()))
            },
            Err(error) => {
                self.resetable_reader.drop_read_bytes();
                warn!("Error on TCP read: {:?}", error);
                Err(ControlFlow::Break(()))
            },
        };
    }
}

struct ResetableReader<T: Read> {
    buf_reader: BufReader<T>,
    buf: VecDeque<u8>,
    fill_len: usize,
    read_len: usize,
}

impl<T: Read> ResetableReader<T> {
    fn new(inner: T) -> Self {
        return Self {
            buf_reader: BufReader::new(inner),
            buf: VecDeque::new(),
            fill_len: 0,
            read_len: 0,
        };
    }

    fn reset_cursor(&mut self) {
        self.read_len = 0;
    }

    fn drop_read_bytes(&mut self) {
        let buf = &self.buf;
        warn!("Buf after drop: {buf:?}");
    }
}

impl<T: Read> Read for &mut ResetableReader<T> {

    fn read(&mut self, read_buf: &mut [u8]) -> io::Result<usize> {

        warn!("Buf: {:?}", self.buf);

        let unread_bytes_in_buf = self.fill_len - self.read_len;
        let bytes_needed_from_tcp_stream = read_buf.len() - unread_bytes_in_buf;
        
        if bytes_needed_from_tcp_stream > self.buf.len() - self.fill_len {
            self.buf.resize(self.fill_len + bytes_needed_from_tcp_stream, 0);
        }

        let slice = self.buf.make_contiguous();

        if bytes_needed_from_tcp_stream > 0 {

            //Need to read bytes from the TcpStream
            let end = bytes_needed_from_tcp_stream + self.fill_len;
            let slice_to_read_into = &mut slice[self.fill_len..end];

            let result = self.buf_reader.read(slice_to_read_into);

            match result {
                Ok(read_len) => {
                    warn!("Buf after read: {slice:?}");
                    self.fill_len += read_len;              
                },

                //TODO: generalize the errors
                Err(error) if error.kind() == io::ErrorKind::TimedOut || error.kind() == io::ErrorKind::WouldBlock => {
                    return Err(Error::from(io::ErrorKind::TimedOut));
                }
                Err(_) => return result,
            }

        }

        //Now, the bytes have already been buffered

        //TODO: start here:
        let bytes_available = slice.len() - self.read_len;
        let len_to_read = min(bytes_available, read_buf.len());

        let slice_to_read_into = &mut read_buf[0..len_to_read];

        slice_to_read_into.copy_from_slice(&mut slice[self.read_len..(self.read_len + len_to_read)]);

        self.read_len += len_to_read;


        let result =Ok(len_to_read);


        warn!("READ\nResult: {result:?}\nread_buf: {read_buf:?}");
        return result;
    }
}

/* 
impl<T: Read> Read for &ResetableReader<T> {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut x = *self;
        return x.read(buf);
    }

}
    */