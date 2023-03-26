use std::{thread, time, io, process};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use log::{error, info, LevelFilter};
use commons::factory::{FactoryTrait, RealFactory};
use commons::logging::LoggingConfigBuilder;
use crate::client::ClientCoreEvent::Connect;
use crate::gamemanager::RenderReceiver;
use crate::simplegame::{SimpleInput, SimpleState, SimpleInputEvent, SimpleInputEventHandler, SimpleWindow, SimpleServerInput, SimpleGameImpl};
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::threading::eventhandling::EventSenderTrait;
use commons::time::TimeDuration;
use crate::interface::RealGameFactory;
use crate::server::ServerCoreEvent;

mod simplegame;
mod messaging;
mod server;
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

    LoggingConfigBuilder::new()
        .add_console_appender()
        .add_file_appender(log_file_path)
        .init(LevelFilter::Info);

    info!("args: {:?}", args);

    let mut server_core_sender_option = None;
    let mut render_receiver_option = None;

    //This unused value keeps the render receiver alive
    let mut unused_render_receiver_option = None;
    let mut client_core_join_handle_option = None;

    let factory = RealFactory::new();

    //TODO: clean this up

    if run_server {

        let server_core_thread_builder = factory.new_thread_builder()
            .name("ServerCore")
            .build_channel_for_event_handler::<server::ServerCore<RealGameFactory<SimpleGameImpl>>>();

        let server_core  = server::ServerCore::<RealGameFactory<SimpleGameImpl>>::new(factory.clone(), server_core_thread_builder.get_sender().clone());

        if let Err(error) = server_core_thread_builder.get_sender().send_event(ServerCoreEvent::StartListenerEvent) {
            error!("{:?}", error);
            return;
        }

        server_core_sender_option = Some(server_core_thread_builder.spawn_event_handler(server_core, AsyncJoin::log_async_join).unwrap());
    }

    if run_client {

        let client_core_thread_builder = factory.new_thread_builder()
            .name("ClientCore")
            .build_channel_for_event_handler::<client::ClientCore<RealGameFactory<SimpleGameImpl>>>();

        let (render_receiver_sender, render_receiver) = RenderReceiver::<RealFactory, SimpleGameImpl>::new(factory.clone());

        render_receiver_option = Some(render_receiver);

        client_core_thread_builder.get_sender().send_event(Connect(render_receiver_sender)).unwrap();

        let sender_clone = client_core_thread_builder.get_sender().clone();

        client_core_join_handle_option = Some(
            client_core_thread_builder.spawn_event_handler(
                client::ClientCore::<RealGameFactory<SimpleGameImpl>>::new(
                    factory.clone(),
                    Ipv4Addr::from_str("127.0.0.1").unwrap(),
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

        let (render_receiver_sender, render_receiver) = RenderReceiver::<RealFactory, SimpleGameImpl>::new(factory.clone());
        unused_render_receiver_option = Some(render_receiver);

        server_core_sender_option.as_ref().unwrap().send_event(ServerCoreEvent::StartGameEvent(render_receiver_sender)).unwrap();

        if !run_client {
            let tmp = unused_render_receiver_option;
            unused_render_receiver_option = render_receiver_option;
            render_receiver_option = tmp;
        }
    }

    let client_window = SimpleWindow::new(factory.clone(), window_name, render_receiver_option.unwrap(), client_core_join_handle_option);
    client_window.run();
}
