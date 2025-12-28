use {
    crate::{
        simplegameimpl::SimpleGameImpl,
        simplestate::SimpleState,
    },
    engine_core::{
        Smooth,
        SmoothArguments,
    },
};

pub struct SimpleSmooth;

impl Smooth for SimpleSmooth {
    type Game = SimpleGameImpl;
    type SmoothedState = SimpleState;

    fn to_smooth_result(state: &SimpleState) -> Self::SmoothedState {
        state.clone()
    }

    fn smooth(arg: &SmoothArguments<Self>) -> Self::SmoothedState {
        SimpleState::interpolate(arg.last_result, arg.future_state, arg.weight)
    }
}
