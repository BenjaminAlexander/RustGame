use std::thread::Builder;

use log::info;

use crate::threading::Thread;

pub(crate) fn spawn_thread<T: Thread>(
    thread_name: String,
    thread: T,
) -> std::io::Result<()> {
    let builder = Builder::new().name(thread_name.clone());

    builder.spawn(|| {
        info!("Thread Starting");

        thread.run();

        info!("Thread Ending");
    })?;

    return Ok(());
}
