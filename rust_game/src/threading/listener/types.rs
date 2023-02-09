use std::ops::ControlFlow;
use crate::threading::eventhandling;
use crate::threading::listener::{ListenedOrDidNotListen, ListenerState, ListenerTrait};

pub(crate) type JoinHandle<T> = eventhandling::JoinHandle<ListenerState<T>>;

pub(crate) type ListenResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, ListenedOrDidNotListen<T>>;

pub(crate) type ListenerEventResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, T>;
