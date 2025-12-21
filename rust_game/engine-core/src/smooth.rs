use commons::time::TimeValue;

use crate::{GameTrait, state_channel::CurrentStates};

pub struct ExtrapolationArguments<'a, T: Smooth + ?Sized> {
    pub last_result: &'a T::SmoothResult,
    pub future_state: &'a <T::Game as GameTrait>::State,
    pub weight: f64
}

pub trait Smooth {
    type Game: GameTrait;
    type SmoothResult;

    fn to_smooth_result(state: &<Self::Game as GameTrait>::State) -> Self::SmoothResult;

    fn smooth(arg: &ExtrapolationArguments<Self>) -> Self::SmoothResult;
}