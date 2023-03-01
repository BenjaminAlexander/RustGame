use std::ops::ControlFlow;
use crate::threading::listener::{ListenedOrDidNotListen, ListenerTrait};

pub(crate) type ListenResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, ListenedOrDidNotListen<T>>;

pub(crate) type ListenerEventResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, T>;
