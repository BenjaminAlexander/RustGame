#![deny(rustdoc::broken_intra_doc_links)]
use {
    crate::{
        simplegameimpl::SimpleGameImpl,
        simpleinput::SimpleInput,
        simpleinputevent::SimpleInputEvent,
        simpleinputeventhandler::SimpleInputEventHandler,
        simplestate::*,
        simplewindow::SimpleWindow,
    },
    commons::{
        logging::LoggingConfigBuilder,
        real_time::Factory,
        time::TimeDuration,
    },
    engine_core::{
        Client,
        Server,
    },
    log::{
        LevelFilter,
        error,
        info,
    },
    std::{
        backtrace::Backtrace,
        io::stdin,
        panic,
        path::PathBuf,
        process,
    },
};

mod bullet;
mod character;
mod simple_smooth;
mod simplegameimpl;
mod simpleinput;
mod simpleinputevent;
mod simpleinputeventhandler;
mod simpleserverinput;
mod simplestate;
mod simplewindow;

pub fn main() {
    let mut run_client = None;
    let mut window_name: String = String::from("Server");

    let args: Vec<String> = std::env::args().collect();

    let mut current_arg = 1;

    while current_arg < args.len() {
        let arg = &args[current_arg];

        match arg.as_str() {
            "-s" => {
                if let Some(true) = run_client {
                    panic!("This execution cannot run both a server and a client");
                }

                run_client = Some(false);
                current_arg = current_arg + 1;
            }
            "-c" => {
                if let Some(false) = run_client {
                    panic!("This execution cannot run both a server and a client");
                }

                run_client = Some(true);
                window_name = String::from(&args[current_arg + 1]);
                current_arg = current_arg + 2;
            }
            _ => {
                panic!("Unrecognized argument: {:?}", arg);
            }
        }
    }

    let mut log_file_path = PathBuf::new();
    log_file_path.push("log");
    log_file_path.push(format!("{}-{}.log", window_name, process::id()));

    LoggingConfigBuilder::new()
        .add_console_appender()
        .add_file_appender(log_file_path)
        .init(LevelFilter::Info);

    //TODO: move panic hook into util function
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!(
            "panic occurred: {panic_info}\n{}",
            Backtrace::force_capture()
        );
        orig_hook(panic_info);
    }));

    info!("args: {:?}", args);

    let factory = Factory::new();

    if let Some(true) = run_client {
        let (client, render_receiver) = Client::<SimpleGameImpl>::new(factory.clone()).unwrap();

        let client_window = SimpleWindow::new(window_name, render_receiver, Some(client.clone()));

        client_window.run();

        client.stop().unwrap();
    } else {
        let (server, state_receiver) = Server::<SimpleGameImpl>::new(factory.clone()).unwrap();

        info!("Hit enter to start the game.");
        let stdin = stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        info!("line: {:?}", line);

        server.start_game().unwrap();

        let server_window = SimpleWindow::new(window_name, state_receiver, None);

        server_window.run();

        server.stop().unwrap();
    }
}
