extern crate bws;
extern crate dotenv;
extern crate iron;
#[macro_use]
extern crate log;
extern crate logger;
extern crate mount;
extern crate router;
extern crate simplelog;
extern crate ws;

use std::env;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use dotenv::dotenv;
use iron::Iron;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

use bws::heartbeat::communication::Message as HeartbeatMessage;
use bws::heartbeat::Heartbeat;
use bws::server;
use bws::simulation::communication::Message as TeamsMessage;
use bws::simulation::Simulation;
use bws::websocket::communication::Message as WsMessage;
use bws::websocket::WebSocketUpdate;

fn main() {
    dotenv().ok();
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap();

    info!("Logger configured");

    let (simulation_tx, simulation_rx): (Sender<TeamsMessage>, Receiver<TeamsMessage>) = channel();
    let (heartbeat_tx, heartbeat_rx): (Sender<HeartbeatMessage>, Receiver<HeartbeatMessage>) =
        channel();
    let (ws_tx, ws_rx): (Sender<WsMessage>, Receiver<WsMessage>) = channel();

    let simulation_heartbeat_tx = heartbeat_tx.clone();
    let simulation_ws_tx = ws_tx.clone();
    let simulation_thread = thread::Builder::new()
        .name("simulation".to_string())
        .spawn(move || {
            info!("starting simulation");
            let mut simulation = Simulation::new();

            simulation.start(simulation_rx, simulation_heartbeat_tx, simulation_ws_tx);
        })
        .unwrap();

    let iron_simulation_tx = simulation_tx.clone();
    let iron_thread = thread::Builder::new()
        .name("iron".to_string())
        .spawn(move || {
            let server_address = env::var("address").expect("\"address\" in environment variables");
            info!("starting server at {}", server_address);

            Iron::new(server::chain(&iron_simulation_tx))
                .http(server_address)
                .unwrap();
        })
        .unwrap();

    let heartbeat_simulation_tx = simulation_tx.clone();
    let heartbeat_thread = thread::Builder::new()
        .name("heartbeat".to_string())
        .spawn(move || {
            let sleep_duration_value = u64::from_str_radix(
                &env::var("heartbeat_sleep_duration")
                    .expect("\"heartbeat_sleep_duration\" in environment variables"),
                10,
            )
            .expect("\"heartbeat_sleep_duration\" to be u64");
            let sleep_duration = Duration::from_secs(sleep_duration_value);

            let mut heartbeat = Heartbeat::new(sleep_duration, heartbeat_rx, heartbeat_simulation_tx);
            info!("starting heartbeat monitor");

            heartbeat.monitor();
        })
        .unwrap();

    let ws_thread = thread::Builder::new()
        .name("socket".to_string())
        .spawn(move || {
            info!("starting websocket communication");
            let socket_address = env::var("socket").expect("\"socket\" in environment variables");
            let ws_update = WebSocketUpdate::new(socket_address);

            ws_update.dispatch(ws_rx)
        })
        .unwrap();

    let clock_simulation_tx = simulation_tx.clone();
    let clock_thread = thread::Builder::new()
        .name("clock".to_string())
        .spawn(move || {
            info!("starting clock");
            let tick_representation = env::var("tick").expect("\"tick\" in environment variables");
            let tick_duration_value =
                u64::from_str_radix(&tick_representation, 10).expect("\"tick\" to be u64");
            let tick_duration = Duration::from_millis(tick_duration_value);

            loop {
                thread::sleep(tick_duration);
                if let Err(error) = clock_simulation_tx.send(TeamsMessage::Tick) {
                    error!("Could not send tick message: {}", error);
                }
            }
        })
        .unwrap();

    clock_thread.join().unwrap();
    iron_thread.join().unwrap();
    heartbeat_thread.join().unwrap();
    ws_thread.join().unwrap();
    simulation_thread.join().unwrap();
}
