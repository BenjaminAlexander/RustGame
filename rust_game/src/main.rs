use std::{thread, time, io, process};
use std::path::PathBuf;
use log::{error, info};
use crate::simplegame::{SimpleInput, SimpleState, SimpleInputEvent, SimpleInputEventHandler, SimpleWindow, SimpleServerInput, SimpleGameImpl};
use crate::threading::ChannelThread;
use crate::gametime::TimeDuration;

mod simplegame;
mod messaging;
mod server;
mod logging;
mod threading;
mod interface;
mod gametime;
mod util;
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
    let mut client_core_sender_option = None;

    if run_server {
        let server_core  = server::ServerCore::<SimpleGameImpl>::new();

        let (server_core_sender, server_core_builder) = server_core.build();

        if let Err(error) = server_core_sender.start_listener() {
            error!("{:?}", error);
            return;
        }

        server_core_builder.name("ServerCore").start().unwrap();

        server_core_sender_option = Some(server_core_sender);
    }

    if run_client {

        let client_core = client::ClientCore::<SimpleGameImpl>::new("127.0.0.1");

        let (client_core_sender, client_core_builder) = client_core.build();

        render_receiver_option = Some(client_core_sender.connect());
        client_core_sender_option = Some(client_core_sender);
        client_core_builder.name("ClientCore").start().unwrap();

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

        unused_render_receiver_option = Some(server_core_sender_option.as_ref().unwrap().start_game());

        if !run_client {
            let tmp = unused_render_receiver_option;
            unused_render_receiver_option = render_receiver_option;
            render_receiver_option = tmp;
        }
    }


    let client_window = SimpleWindow::new(window_name, render_receiver_option.unwrap(), client_core_sender_option);
    client_window.run();
}
