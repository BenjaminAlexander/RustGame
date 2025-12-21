use engine_core::{ExtrapolationArguments, Smooth};

use crate::{simplegameimpl::SimpleGameImpl, simplestate::SimpleState};

pub struct SimpleSmooth;

impl Smooth for SimpleSmooth {
    type Game = SimpleGameImpl;
    type SmoothResult = SimpleState;

    fn to_smooth_result(state: &SimpleState) -> Self::SmoothResult {
        state.clone()
    }

    fn smooth(arg: &ExtrapolationArguments<Self>) -> Self::SmoothResult {
        SimpleState::interpolate(arg.last_result, arg.future_state, arg.weight)
    }
}