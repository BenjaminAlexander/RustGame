use crate::interface::State;
use crate::interface::interpolationarg::InterpolationArg;

pub trait Interpolate<StateType: State, InterpolatedType> {

    fn interpolate(first: &StateType, second: &StateType, arg: &InterpolationArg) -> InterpolatedType;
}