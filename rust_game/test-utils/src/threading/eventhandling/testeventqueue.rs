use commons::threading::eventhandling::EventHandlerTrait;

struct TestEventQueue<T: EventHandlerTrait> {
    event_handler: T
}