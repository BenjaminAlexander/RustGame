use crate::ThreadBuilderTrait;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::thread::Thread;
use crate::threading::eventhandling::Sender;
use crate::threading::ThreadBuilder as BaseThreadBuilder;
use crate::threading::eventhandling::joinhandle::JoinHandle;

pub struct ThreadBuilder<T: EventHandlerTrait> {
    pub(super) sender: Sender<T>,
    pub(super) builder: BaseThreadBuilder<Thread<T>>
}

impl<T: EventHandlerTrait> ThreadBuilder<T> {

    pub fn get_sender(&self) -> &Sender<T> { &self.sender }

}

impl<T: EventHandlerTrait> ThreadBuilderTrait for ThreadBuilder<T> {
    type StartResultType = std::io::Result<JoinHandle<T>>;

    fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name);
        return self;
    }

    fn start(self) -> std::io::Result<JoinHandle<T>> {
        let join_handle = self.builder.start()?;

        return Result::Ok(JoinHandle {
            sender: self.sender,
            join_handle
        });
    }
}