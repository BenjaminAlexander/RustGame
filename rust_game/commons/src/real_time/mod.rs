mod eventhandlertrait;
mod event_handler_builder;
mod event_handler_stopper;
mod event_or_stop_thread;
mod event_sender;
mod factorytrait;
mod real;
mod realfactory;
mod receive_meta_data;
mod receiver;
mod send_meta_data;
mod sender;

pub mod simulation;

pub use self::event_handler_builder::EventHandlerBuilder;
pub use self::event_handler_stopper::EventHandlerStopper;
pub use self::event_sender::EventSender;
pub use self::factorytrait::FactoryTrait;
pub use self::receive_meta_data::ReceiveMetaData;
pub use self::realfactory::RealFactory;
pub use self::receiver::Receiver;
pub use self::send_meta_data::SendMetaData;
pub use self::sender::Sender;
pub use self::eventhandlertrait::EventHandlerTrait;
pub use self::eventhandlertrait::EventHandleResult;

//TODO: hide this
pub use self::event_or_stop_thread::EventOrStopThread;