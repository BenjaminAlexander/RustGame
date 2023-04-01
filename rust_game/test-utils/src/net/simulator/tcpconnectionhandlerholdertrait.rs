use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::{AsyncJoin, AsyncJoinCallBackTrait, ThreadBuilder};

pub trait TcpConnectionHandlerHolderTrait {

}

pub fn new<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
>(
    thread_builder: ThreadBuilder<Factory>,
    connection_handler: TcpConnectionHandler,
    join_call_back: AsyncJoinCallBack
) -> Box<dyn TcpConnectionHandlerHolderTrait + Send> {

    let holder = Holder {
        connection_handler,
        thread_builder,
        join_call_back
    };

    let tcp_connection_handler_holder = TcpConnectionHandlerHolder {
        holder: Some(holder)
    };

    return Box::new(tcp_connection_handler_holder);
}

struct TcpConnectionHandlerHolder<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> {
    holder: Option<Holder<Factory, TcpConnectionHandler, AsyncJoinCallBack>>
}

struct Holder<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> {
    connection_handler: TcpConnectionHandler,
    thread_builder: ThreadBuilder<Factory>,
    join_call_back: AsyncJoinCallBack
}

impl<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> Drop for TcpConnectionHandlerHolder<Factory, TcpConnectionHandler, AsyncJoinCallBack> {
    fn drop(&mut self) {
        if let Some(holder) = self.holder.take() {
            let async_join = AsyncJoin::new(holder.thread_builder, holder.connection_handler);
            holder.join_call_back.join(async_join);
        }
    }
}

impl<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>,
    AsyncJoinCallBack: AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
> TcpConnectionHandlerHolderTrait for TcpConnectionHandlerHolder<Factory, TcpConnectionHandler, AsyncJoinCallBack> {

}