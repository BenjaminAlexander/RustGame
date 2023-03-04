use std::{thread, time, io, process};
use std::path::PathBuf;
use log::{error, info};
use crate::client::ClientCoreEvent::Connect;
use crate::gamemanager::RenderReceiver;
use crate::simplegame::{SimpleInput, SimpleState, SimpleInputEvent, SimpleInputEventHandler, SimpleWindow, SimpleServerInput, SimpleGameImpl};
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::time::TimeDuration;
use crate::server::ServerCoreEvent;

mod simplegame;
mod messaging;
mod server;
mod logging;
mod interface;
mod gametime;
mod client;
mod gamemanager;

pub fn main() {

    let args: Vec<String> = std::env::args().collect();

    let mut run_client = true;
    let mut run_server = true;
    let mut window_name:String = String::from("Server");

    if args.len() >= 2  {
        if args[1].eq("-s") {
            run_client = false;

        } else if args[1].eq("-c") {
            run_server = false;
        }
    }

    if args.len() > 2  {
        window_name = String::from(&args[2]);
    }

    let mut log_file_path = PathBuf::new();
    log_file_path.push("log");
    log_file_path.push(format!("{}-{}.log", window_name, process::id()));

    logging::init_logging(log_file_path);

    info!("args: {:?}", args);

    let mut server_core_sender_option = None;
    let mut render_receiver_option = None;

    //This unused value keeps the render receiver alive
    let mut unused_render_receiver_option = None;
    let mut client_core_join_handle_option = None;

    if run_server {

        let server_core_thread_builder = ThreadBuilder::new()
            .name("ServerCore")
            .build_channel_for_event_handler::<server::ServerCore<SimpleGameImpl>>();

        let server_core  = server::ServerCore::<SimpleGameImpl>::new(server_core_thread_builder.get_sender().clone());

        if let Err(error) = server_core_thread_builder.get_sender().send_event(ServerCoreEvent::StartListenerEvent) {
            error!("{:?}", error);
            return;
        }

        server_core_sender_option = Some(server_core_thread_builder.spawn_event_handler(server_core, AsyncJoin::log_async_join).unwrap());
    }

    if run_client {

        let client_core_thread_builder = ThreadBuilder::new()
            .name("ClientCore")
            .build_channel_for_event_handler::<client::ClientCore<SimpleGameImpl>>();

        let (render_receiver_sender, render_receiver) = RenderReceiver::<SimpleGameImpl>::new();

        render_receiver_option = Some(render_receiver);

        client_core_thread_builder.get_sender().send_event(Connect(render_receiver_sender)).unwrap();

        let sender_clone = client_core_thread_builder.get_sender().clone();

        client_core_join_handle_option = Some(
            client_core_thread_builder.spawn_event_handler(
                client::ClientCore::<SimpleGameImpl>::new(
                    "127.0.0.1",
                    sender_clone
                ),
                AsyncJoin::log_async_join
            ).unwrap()
        );

        let millis = time::Duration::from_millis(1000);
        thread::sleep(millis);
    }

    if run_server {

        if !run_client {
            info!("Hit enter to start the game.");
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();

            info!("line: {:?}", line);
        }

        let (render_receiver_sender, render_receiver) = RenderReceiver::<SimpleGameImpl>::new();
        unused_render_receiver_option = Some(render_receiver);

        server_core_sender_option.as_ref().unwrap().send_event(ServerCoreEvent::StartGameEvent(render_receiver_sender)).unwrap();

        if !run_client {
            let tmp = unused_render_receiver_option;
            unused_render_receiver_option = render_receiver_option;
            render_receiver_option = tmp;
        }
    }

    let client_window = SimpleWindow::new(window_name, render_receiver_option.unwrap(), client_core_join_handle_option);
    client_window.run();
}
