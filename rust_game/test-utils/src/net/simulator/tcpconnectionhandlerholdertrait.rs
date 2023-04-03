use std::ops::ControlFlow;
use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::{AsyncJoin, AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::{Receiver, TryRecvError};
use commons::threading::eventhandling::EventOrStopThread;
use crate::net::{ChannelTcpReader, ChannelTcpWriter};
use crate::singlethreaded::SingleThreadedFactory;

pub trait TcpConnectionHandlerHolderTrait: Send {

    fn on_send(self: Box<Self>) -> Option<Box<dyn TcpConnectionHandlerHolderTrait>>;

    fn on_connection(self: Box<Self>, writer: ChannelTcpWriter, reader: ChannelTcpReader) -> Option<Box<dyn TcpConnectionHandlerHolderTrait>>;

    fn stop(self: Box<Self>);
}

pub fn new<
    TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<SingleThreadedFactory, TcpConnectionHandler>
>(
    thread_builder: ThreadBuilder<SingleThreadedFactory>,
    receiver: Receiver<SingleThreadedFactory, EventOrStopThread<()>>,
    connection_handler: TcpConnectionHandler,
    join_call_back: AsyncJoinCallBack
) -> Box<dyn TcpConnectionHandlerHolderTrait> {

    let tcp_connection_handler_holder = TcpConnectionHandlerHolder {
        receiver,
        connection_handler,
        thread_builder,
        join_call_back
    };

    return Box::new(tcp_connection_handler_holder);
}

struct TcpConnectionHandlerHolder<
    TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<SingleThreadedFactory, TcpConnectionHandler>
> {
    receiver: Receiver<SingleThreadedFactory, EventOrStopThread<()>>,
    connection_handler: TcpConnectionHandler,
    thread_builder: ThreadBuilder<SingleThreadedFactory>,
    join_call_back: AsyncJoinCallBack
}

impl<
    TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<SingleThreadedFactory, TcpConnectionHandler>
> TcpConnectionHandlerHolder<TcpConnectionHandler, AsyncJoinCallBack> {

    fn join(self) {
        let async_join = AsyncJoin::new(self.thread_builder, self.connection_handler);
        self.join_call_back.join(async_join);
    }

}

impl<
    TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<SingleThreadedFactory, TcpConnectionHandler>
> TcpConnectionHandlerHolderTrait for TcpConnectionHandlerHolder<TcpConnectionHandler, AsyncJoinCallBack> {

    fn on_send(mut self: Box<Self>) -> Option<Box<dyn TcpConnectionHandlerHolderTrait>> {

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

    fn on_connection(mut self: Box<Self>, writer: ChannelTcpWriter, reader: ChannelTcpReader) -> Option<Box<dyn TcpConnectionHandlerHolderTrait>> {

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