use crate::spoof_sensor::SpoofSensor;
use clap::Parser;
use package_parser::{DataReceiver, Package, PackageV1};
use rusqlite::Connection;
use serialport::new;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::spawn;
use std::time::Duration;
use wired_arduino::WiredArduino;

mod spoof_sensor;
mod wired_arduino;

#[derive(Parser)]
struct Cli {
    database_path: String,
    port: Option<String>,
    baud_rate: Option<u32>,

    #[clap(long, action)]
    spoof: bool,
}

fn main() {
    let cli = Cli::parse();

    let conn = Connection::open(&cli.database_path).expect("database connection failed");

    conn.execute_batch("PRAGMA journal_mode=WAL")
        .expect("failed to execute PRAGMA");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS data (
        id          INTEGER PRIMARY KEY,
        sensor      INTEGER,
        timestamp   INTEGER,
        value       INTEGER
    )",
        [],
    )
    .expect("failed to create data table");

    let (tx, rx) = mpsc::channel();

    for receiver in get_receivers(cli) {
        let tx = tx.clone();
        spawn(move || read_from_stream(receiver, tx, true));
    }

    let mut statement = conn
        .prepare("INSERT INTO data (sensor, timestamp, value) VALUES (?, ?, ?)")
        .expect("failed to prepare statement");

    for package in rx {
        statement
            .execute([package.sensor, package.timestamp, package.value])
            .expect("failed to execute statement");
    }
}

fn get_receivers(cli: Cli) -> Vec<Box<dyn DataReceiver>> {
    let mut vec = vec![];

    if let (Some(port_name), Some(baud_rate)) = (cli.port, cli.baud_rate) {
        let port = new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .expect("failed to open serial port");

        let wired_arduino = WiredArduino::new(port);

        vec.push(Box::new(wired_arduino) as Box<dyn DataReceiver>);
    }

    if cli.spoof {
        for x in 1..=4 {
            vec.push(Box::new(SpoofSensor::new(x)));
        }
    }

    vec
}

fn read_from_stream(mut receiver: Box<dyn DataReceiver>, tx: Sender<PackageV1>, debug: bool) {
    let mut buffer = Vec::new();
    while let Some(next) = receiver.get_next_byte() {
        buffer.push(next);
        if let Some(package) = Package::try_from_buffer(&buffer) {
            if let Some(package_v1) = PackageV1::try_from(package, debug) {
                tx.send(package_v1)
                    .expect("failed to send package via channel");

                buffer.clear();
            }
        }
    }
}
