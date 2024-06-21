use crate::factory::FactoryTrait;
use std::ops::ControlFlow;

pub trait TcpConnectionHandlerTrait: Send + 'static {
    type Factory: FactoryTrait;

    fn on_connection(
        &mut self,
        tcp_sender: <Self::Factory as FactoryTrait>::TcpWriter,
        tcp_receiver: <Self::Factory as FactoryTrait>::TcpReader,
    ) -> ControlFlow<()>;
}
