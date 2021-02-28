use crate::interface::State;
use crate::interface::interpolationarg::InterpolationArg;
use crate::interface::interpolationresult::InterpolationResult;
use crate::messaging::InitialInformation;

pub trait Interpolate<StateType: State, InterpolationResultType: InterpolationResult>: Send + 'static {

    fn interpolate(initial_information: &InitialInformation<StateType>,
                   first: &StateType,
                   second: &StateType,
                   arg: &InterpolationArg) -> InterpolationResultType;
}