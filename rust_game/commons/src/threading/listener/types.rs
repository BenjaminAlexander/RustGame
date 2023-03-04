use std::ops::ControlFlow;
use crate::threading::listener::{ListenedOrDidNotListen, ListenerTrait};

pub type ListenResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, ListenedOrDidNotListen<T>>;

pub type ListenerEventResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, T>;
