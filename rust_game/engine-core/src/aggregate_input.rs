use crate::GameTrait;

/// Defines how the client will aggregate a steam of input events
/// occuring at any time into an input aggregate at each frame index.
///
/// An input event is an event that can be submitted to the client at any time
/// during the sequence of frames.  
///
/// An input aggregate is the aggregation of all input events that occur between
/// the time a frame occurs and the next frame.
///
/// Any time a input event is received, the aggregator should return a new
/// aggregate if the value of the aggregate is changed from its previously
/// returned value.  This value will supersede the previously returned values for
/// the current frame.  Each value of the aggregate that is returned, could
/// potentially become the final value of the input aggregate for the frame if
/// no additional input events are received during the current frame.
pub trait AggregateInput: Send {
    type Game: GameTrait;

    /// Creates a new Aggregator
    fn new() -> Self;

    /// Aggregates a new input.  As inputs are streamed into the local client,
    /// this function will be called to aggregate them into a single input aggregation
    /// for the current frame.  Subsequent calls to this function without a call of
    /// [AggregateInput::start_next_frame] should return superseding values of the
    /// aggregation for the current frame if the value is changed by the newly
    /// aggregated event.
    ///
    /// If this input changes the value of the current input aggregation, this should
    /// return some input aggregate.  The client will then apply the input aggregation
    /// to the local current state and send it to the server.  This will supersede
    /// any previously returned aggregates for this frame.
    ///
    /// If the event does not effect the value of the aggregate, this should return
    /// none.  Since the pervious value of the input aggregate has already been
    /// returned, nothing is done by the client.
    ///
    /// The factional frame duration is the time within the frame that this input
    /// event occured.  It's value will be between 0.0 and 1.0 and is the factor
    /// of the frame duration past the occurance of the current frame where the
    /// input occured in time.
    ///
    /// TODO: make type for fractional frame duration
    fn aggregate_input_event(
        &mut self,
        fractional_frame_duration: f64,
        input_event: <Self::Game as GameTrait>::InputEvent,
    ) -> Option<<Self::Game as GameTrait>::InputAggregate>;

    /// Resets the aggregator at the start of a new frame.  This should return the
    /// initial input aggregate for the new frame.  This aggregate could become
    /// the final aggregate if no further input events are received.
    fn start_next_frame(&mut self) -> <Self::Game as GameTrait>::InputAggregate;
}
