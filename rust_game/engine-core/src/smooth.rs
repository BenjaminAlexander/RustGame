use crate::Play;

/// The arguments for [Smooth::smooth]
pub struct SmoothArguments<'a, T: Smooth + ?Sized> {
    /// The previously created result.
    ///
    /// This result is typically for a time before [SmoothArguments::future_state],
    /// but in some circumstances this result can actually be after.  In the case
    /// where [SmoothArguments::last_result] is after [SmoothArguments::future_state],
    /// [SmoothArguments::weight] will be negetive.
    pub last_result: &'a T::SmoothedState,

    /// A future known state.
    ///
    /// This state is typically for a time after [SmoothArguments::last_result],
    /// but in some circumstances this state can actually be before.  In the case
    /// where [SmoothArguments::future_state] is before [SmoothArguments::last_result],
    /// [SmoothArguments::weight] will be negetive.
    pub future_state: &'a <T::Game as Play>::State,

    /// The weight between [SmoothArguments::last_result] and [SmoothArguments::future_state]
    /// that the new [Smooth::SmoothedState] should be created at.
    ///  
    /// * A value of 0.0 indicates that the new result should be equivalent to
    /// [SmoothArguments::last_result].  
    /// * A value of 1.0 indicates that the new result should be equivalent to
    /// [SmoothArguments::future_state].
    /// * A value greater than 1.0 indicates that the new result should be
    /// extrapolated past [SmoothArguments::future_state].
    /// * A value less than 0.0 indicates that [SmoothArguments::last_result] is
    /// actually after [SmoothArguments::future_state] and that the new result
    /// should be extrapolated past [SmoothArguments::last_result]
    ///
    /// Implementing [Smooth::smooth] as linear interpolation/extrapolation will
    /// elegantly handle all of these cases.
    pub weight: f64,
}

/// Defines how errors between the last [SmoothedState][Smooth::SmoothedState]
/// and a future [State][Play::State] are corrected
pub trait Smooth {
    type Game: Play;
    type SmoothedState;

    /// Convert a [Play::State] to a [Smooth::SmoothedState].  This function
    /// is used to produce the very first [Smooth::SmoothedState].
    fn to_smooth_result(state: &<Self::Game as Play>::State) -> Self::SmoothedState;

    /// Calculate a new [Smooth::SmoothedState] at [SmoothArguments::weight] between
    /// [SmoothArguments::last_result] and [SmoothArguments::future_state].
    ///
    /// [SmoothArguments::last_result] and [SmoothArguments::future_state] can be
    /// thought of as two points.  A sensable implementation of smooth can be
    /// thought of as performing a linear interpolation/extrapolation between/from
    /// those two points.  [SmoothArguments::weight] is the ratio between or past
    /// either point where the returned [Smooth::SmoothedState] should be located.
    fn smooth(arg: &SmoothArguments<Self>) -> Self::SmoothedState;
}
