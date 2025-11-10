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
    join_call_back: impl AsyncJoinCallBackTrait<()>,
) -> std::io::Result<()> {
    let builder = Builder::new().name(thread_name.clone());

    builder.spawn(|| {
        info!("Thread Starting");

        let return_value = thread.run();
        let async_join = AsyncJoin::new(thread_name, return_value);
        join_call_back.join(async_join);

        info!("Thread Ending");
    })?;

    return Ok(());
}
