use std::thread::Builder;

use log::info;

use crate::threading::Thread;

pub(crate) fn spawn_thread<T: Thread>(
    thread_name: String,
    thread: T,
    call_back: impl FnOnce(T::ReturnType) + Send + 'static,
) -> std::io::Result<()> {
    let builder = Builder::new().name(thread_name.clone());

    builder.spawn(move || {
        info!("Thread Starting: {}", thread_name);

        let return_value = thread.run();

        info!("Thread function complete. Invoking callback: {}", thread_name);

        (call_back)(return_value);

        info!("Thread Ending: {}", thread_name);
    })?;

    return Ok(());
}
