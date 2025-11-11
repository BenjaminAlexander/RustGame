use std::thread::Builder;

use log::info;

use crate::threading::{
    AsyncJoin,
    AsyncJoinCallBackTrait,
    Thread,
};

pub(crate) fn spawn_thread<T: Thread>(
    thread_name: String,
    thread: T,
    join_call_back: impl AsyncJoinCallBackTrait<T::ReturnType>,
) -> std::io::Result<()> {
    let builder = Builder::new().name(thread_name.clone());

    builder.spawn(move || {
        info!("Thread Starting: {}", thread_name);

        let return_value = thread.run();
        let async_join = AsyncJoin::new(return_value);

        info!("Thread function complete. Invoking callback: {}", thread_name);

        join_call_back.join(async_join);

        info!("Thread Ending: {}", thread_name);
    })?;

    return Ok(());
}
