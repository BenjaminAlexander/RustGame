use commons::factory::FactoryTrait;
use commons::threading::channel::{Channel, SenderTrait, TryRecvError};
use test_utils::singlethreaded::SingleThreadedFactory;
use test_utils::utils::Counter;

#[test]
fn test_sender() {

    let factory = SingleThreadedFactory::new();

    let (sender, mut receiver) = factory.new_channel().take();

    let counter = Counter::new(0);

    let counter_clone = counter.clone();
    sender.clone().set_on_send(move ||{
        counter_clone.increment();
    });

    assert_eq!(0, counter.get());

    sender.send(()).unwrap();
    assert_eq!(1, counter.get());
    assert_eq!((), receiver.try_recv(&factory).unwrap());

    drop(sender);
    assert_eq!(2, counter.get());
    assert_eq!(TryRecvError::Disconnected, receiver.try_recv(&factory).unwrap_err());
}