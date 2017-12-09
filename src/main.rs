extern crate bws;
extern crate dotenv;
extern crate iron;
#[macro_use] extern crate log;
extern crate logger;
extern crate mount;
extern crate router;
extern crate simplelog;
extern crate ws;

use std::env;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use dotenv::dotenv;
use iron::Iron;
use simplelog::{Config, LogLevelFilter, TermLogger, CombinedLogger};
use ws::{WebSocket, Message};

use bws::heartbeat::Heartbeat;
use bws::heartbeat::communication::Message as HeartbeatMessage;
use bws::model::Simulation;
use bws::model::communication::Message as TeamsMessage;
use bws::server;

fn main() {
    dotenv().ok();
    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Info, Config::default()).unwrap(),
        ]
    ).unwrap();

    info!("Logger configured");

    let (simulation_tx, simulation_rx): (Sender<TeamsMessage>, Receiver<TeamsMessage>) = channel();
    let (heartbeat_tx, heartbeat_rx): (Sender<HeartbeatMessage>, Receiver<HeartbeatMessage>) = channel();
    let (ws_tx, ws_rx): (Sender<WsMessage>, Receiver<WsMessage>) = channel();

    let simulation_heartbeat_tx = heartbeat_tx.clone();
    let simulation_thread = thread::spawn(move ||{
        info!("simulation thread started");
        let mut simulation = Simulation::new();

        simulation.start(simulation_rx, simulation_heartbeat_tx);
    });

    let iron_simulation_tx = simulation_tx.clone();
    let iron_thread = thread::spawn(move ||{
        let server_address = env::var("address").expect("\"address\" in environment variables");
        info!("starting server at {}", server_address);

        Iron::new(server::chain(&iron_simulation_tx)).http(server_address).unwrap();
    });

    let heartbeat_simulation_tx = simulation_tx.clone();
    let heartbeat_thread = thread::spawn(move ||{
        let mut heartbeat = Heartbeat::new(heartbeat_rx, heartbeat_simulation_tx);
        info!("starting heartbeat monitor");

        heartbeat.monitor();
    });

    let ws_simulation_tx = simulation_tx.clone();
    let ws_thread = thread::spawn(move ||{
        let socket_address = env::var("socket").expect("\"socket\" in environment variables");
        if let Ok(web_socket) = WebSocket::new(|out: ws::Sender| {
            move |msg: Message| {
                info!("Server got message '{}'. ", msg);
                out.broadcast("")
            }
        }) {
            let sender = web_socket.broadcaster();
            let send_thread = thread::spawn(move||{
                let message = ws_rx.recv().unwrap();
                match message {
                    WsMessage::Default => {
                        sender.send("").unwrap();
                    }
                }
            });
            if let Err(error) = web_socket.listen(socket_address) {
                error!("Websocket could not listen {:?}", error);
            }
            send_thread.join().unwrap();
        } else {
            error!("Failed to create WebSocket");
        }
    });

    iron_thread.join().unwrap();
    heartbeat_thread.join().unwrap();
    ws_thread.join().unwrap();
    simulation_thread.join().unwrap();
}

enum WsMessage {
    Default
}
