use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;

pub trait TcpStreamTrait {
    type T;

    fn read(&self) -> Result<Self::T, DecodeError>;

    fn write(&mut self, t: &Self::T) -> Result<(), EncodeError>;
}