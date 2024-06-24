use crate::factory::FactoryTrait;
use std::ops::ControlFlow;

pub trait TcpConnectionHandlerTrait<Factory: FactoryTrait>: Send + 'static {
    fn on_connection(
        &mut self,
        tcp_sender: Factory::TcpWriter,
        tcp_receiver: Factory::TcpReader,
    ) -> ControlFlow<()>;
}

impl<
        Factory: FactoryTrait,
        U: FnMut(Factory::TcpWriter, Factory::TcpReader) -> ControlFlow<()> + Send + 'static,
    > TcpConnectionHandlerTrait<Factory> for U
{
    fn on_connection(
        &mut self,
        tcp_sender: Factory::TcpWriter,
        tcp_receiver: Factory::TcpReader,
    ) -> ControlFlow<()> {
        return self(tcp_sender, tcp_receiver);
    }
}
