use std::ops::ControlFlow;
use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::{AsyncJoin, AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::Receiver;
use commons::threading::eventhandling::EventOrStopThread;

pub trait TcpConnectionHandlerHolderTrait: Send {
    type Factory: FactoryTrait;

    fn on_send(self: Box<Self>) -> Option<Box<dyn TcpConnectionHandlerHolderTrait<Factory=Self::Factory>>>;

    fn on_connection(self: Box<Self>, writer: <Self::Factory as FactoryTrait>::TcpWriter, reader: <Self::Factory as FactoryTrait>::TcpReader) -> Option<Box<dyn TcpConnectionHandlerHolderTrait<Factory=Self::Factory>>>;

    fn stop(self: Box<Self>);
}

pub fn new<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
>(
    thread_builder: ThreadBuilder<Factory>,
    receiver: Receiver<Factory, EventOrStopThread<()>>,
    connection_handler: TcpConnectionHandler,
    join_call_back: AsyncJoinCallBack
) -> Box<dyn TcpConnectionHandlerHolderTrait<Factory=Factory>> {

    let tcp_connection_handler_holder = TcpConnectionHandlerHolder {
        receiver,
        connection_handler,
        thread_builder,
        join_call_back
    };

    return Box::new(tcp_connection_handler_holder);
}

struct TcpConnectionHandlerHolder<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> {
    receiver: Receiver<Factory, EventOrStopThread<()>>,
    connection_handler: TcpConnectionHandler,
    thread_builder: ThreadBuilder<Factory>,
    join_call_back: AsyncJoinCallBack
}

impl<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> TcpConnectionHandlerHolder<Factory, TcpConnectionHandler, AsyncJoinCallBack> {

    fn join(self) {
        let async_join = AsyncJoin::new(self.thread_builder, self.connection_handler);
        self.join_call_back.join(async_join);
    }

}

impl<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> TcpConnectionHandlerHolderTrait for TcpConnectionHandlerHolder<Factory, TcpConnectionHandler, AsyncJoinCallBack> {
    type Factory = Factory;

    fn on_send(mut self: Box<Self>) -> Option<Box<dyn TcpConnectionHandlerHolderTrait<Factory=Self::Factory>>> {
        match self.receiver.recv() {
            Ok(EventOrStopThread::StopThread) => {
                self.stop();
                None
            }
            Ok(EventOrStopThread::Event(())) => {
                return Some(self);
            }
            Err(_error) => {
                self.stop();
                None
            }
        }
    }

    fn on_connection(mut self: Box<Self>, writer: <Self::Factory as FactoryTrait>::TcpWriter, reader: <Self::Factory as FactoryTrait>::TcpReader) -> Option<Box<dyn TcpConnectionHandlerHolderTrait<Factory=Self::Factory>>> {

        return match self.connection_handler.on_connection(writer, reader) {
            ControlFlow::Continue(()) => Some(self),
            ControlFlow::Break(()) => {
                self.stop();
                None
            }
        };
    }


    fn stop(self: Box<Self>) {
        let async_join = AsyncJoin::new(self.thread_builder, self.connection_handler);
        self.join_call_back.join(async_join);
    }
}