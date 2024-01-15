mod game;
mod gamefactorytrait;
mod initialinformation;
mod interpolationarg;
mod realgamefactory;
mod serverupdatearg;
mod types;
mod updatearg;

pub use self::game::GameTrait;
pub use self::gamefactorytrait::GameFactoryTrait;
pub use self::initialinformation::InitialInformation;
pub use self::interpolationarg::InterpolationArg;
pub use self::realgamefactory::RealGameFactory;
pub use self::serverupdatearg::ServerUpdateArg;
pub use self::updatearg::ClientUpdateArg;

pub(crate) use self::types::EventSender;
pub(crate) use self::types::TcpReader;
pub(crate) use self::types::TcpWriter;
pub(crate) use self::types::UdpSocket;
//TODO: I'm not sure this should be exposed
pub(crate) use self::types::ClientInputEvent;
pub use self::types::Factory;
pub(crate) use self::types::Game;
pub(crate) use self::types::InterpolationResult;
pub(crate) use self::types::Receiver;
pub(crate) use self::types::Sender;
