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

/*
impl<T: TcpConnectionHandlerTrait<Factory=F>, F: FactoryTrait, U: FnMut(Box<<F as FactoryTrait>::TcpWriter>, Box<<F as FactoryTrait>::TcpReader>)> TcpConnectionHandlerTrait for U {

    fn on_connection(
        &mut self,
        tcp_sender: <Self::Factory as FactoryTrait>::TcpWriter,
        tcp_receiver: <Self::Factory as FactoryTrait>::TcpReader,
    ) -> ControlFlow<()> {
        todo!()
    }
    
    type Factory;
}*/
