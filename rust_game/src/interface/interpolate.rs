use crate::interface::State;
use crate::interface::interpolationarg::InterpolationArg;
use crate::interface::interpolationresult::InterpolationResult;

pub trait Interpolate<StateType: State, InterpolationResultType: InterpolationResult>: Send + 'static {

    fn interpolate(first: &StateType, second: &StateType, arg: &InterpolationArg) -> InterpolationResultType;
}