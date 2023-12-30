mod updatearg;
mod interpolationarg;
mod serverupdatearg;
mod game;
mod gamefactorytrait;
mod realgamefactory;
mod types;

pub use self::updatearg::ClientUpdateArg;
pub use self::interpolationarg::InterpolationArg;
pub use self::serverupdatearg::ServerUpdateArg;
pub use self::game::GameTrait;
pub use self::gamefactorytrait::GameFactoryTrait;
pub use self::realgamefactory::RealGameFactory;

pub(crate) use self::types::TcpWriter;
pub(crate) use self::types::TcpReader;
pub(crate) use self::types::UdpSocket;
pub(crate) use self::types::EventSender;
//TODO: I'm not sure this should be exposed
pub use self::types::Factory;
pub(crate) use self::types::Sender;
pub(crate) use self::types::Receiver;
pub(crate) use self::types::Game;
pub(crate) use self::types::InterpolationResult;
pub(crate) use self::types::ClientInputEvent;