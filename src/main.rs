mod asd_b_types;
mod coordiator;
mod flight;
mod flight_helpers;
mod hex_db;
mod sbs1_ingest;
mod state;
mod ui;

use crate::coordiator::run_coordinator;
use crate::hex_db::FlightInfo;
use crate::sbs1_ingest::SSB1StreamEvent;
use crate::state::AppState;
use crate::ui::run_ui;
use std::env;
use std::io::{self};
use std::process;
use tokio::sync;

fn usage(program: &str) {
    eprintln!("Usage: {program} <port> [host]");
    eprintln!("Example: {program} 9000 127.0.0.1");
}

fn parse_args() -> (String, u16) {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "icaRS".to_string());

    let port_str = match args.next() {
        Some(p) => p,
        None => {
            usage(&program);
            process::exit(1);
        }
    };

    let port: u16 = match port_str.parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid port: {port_str}");
            usage(&program);
            process::exit(1);
        }
    };

    let host = args.next().unwrap_or_else(|| "127.0.0.1".to_string());
    (host, port)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AppCommand {
    Quit,
    FetchFlightInfo { hex_id: String },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AppEvent {
    Ingest(SSB1StreamEvent),
    FlightInfoEvent(FlightInfo),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (host, port) = parse_args();
    let address = format!("{host}:{port}");

    let mut app_state = AppState::new();

    eprintln!("Connecting to {address}...");

    let (event_tx, event_rx) = sync::mpsc::unbounded_channel::<AppEvent>();
    let (command_tx, command_rx) = sync::mpsc::unbounded_channel::<AppCommand>();

    tokio::spawn(run_coordinator(address, command_rx, event_tx));

    run_ui(event_rx, command_tx, &mut app_state)?;

    Ok(())
}
