use clap::Parser;
use package_parser::{DataReceiver, Package, PackageV1, SpoofedData};
use rusqlite::Connection;
use serialport::{new, SerialPort};
use std::collections::VecDeque;
use std::io::ErrorKind::TimedOut;
use std::io::Read;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::spawn;
use std::time::Duration;

#[derive(Parser)]
struct Cli {
    database_path: String,
    port: Option<String>,
    baud_rate: Option<u32>,
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

        let wired_arduino = WiredArduino {
            port,
            backlog: VecDeque::new(),
        };

        vec.push(Box::new(wired_arduino) as Box<dyn DataReceiver>);
    }

    let spoofed = SpoofedData::from(&[
        77u8, 87u8, 100u8, 1u8, 0u8, 0u8, 0u8, 12u8, 0u8, 0u8, 0u8, 18u8, 39u8, 0u8, 0u8, 42u8,
        0u8, 0u8, 0u8, 184u8, 1u8, 0u8, 0u8, 10u8, 10u8, 0u8, 77u8, 87u8, 100u8, 1u8, 0u8, 0u8,
        0u8, 12u8, 0u8, 0u8, 0u8, 250u8, 42u8, 0u8, 0u8, 42u8, 0u8, 0u8, 0u8, 165u8, 0u8, 0u8, 0u8,
        10u8, 10u8, 0u8,
    ] as &[u8]);
    vec.push(Box::new(spoofed) as _);

    vec
}

fn read_from_stream(mut receiver: Box<dyn DataReceiver>, tx: Sender<PackageV1>, debug: bool) {
    let mut buffer = Vec::new();
    while let Some(next) = receiver.get_next_byte() {
        buffer.push(next);
        if let Some(package) = Package::try_from_buffer(&buffer, debug) {
            if let Some(package_v1) = PackageV1::try_from(package, debug) {
                tx.send(package_v1)
                    .expect("failed to send package via channel");
            }
        }
    }
}

struct WiredArduino {
    port: Box<dyn SerialPort>,
    backlog: VecDeque<u8>,
}

unsafe impl Send for WiredArduino {}
unsafe impl Sync for WiredArduino {}

impl DataReceiver for WiredArduino {
    fn get_next_byte(&mut self) -> Option<u8> {
        loop {
            let mut buffer = [0u8; 1024];

            match self.port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    self.backlog = VecDeque::from(buffer[..n].to_vec());
                    break;
                }
                Ok(_) => {}
                Err(ref err) if err.kind() == TimedOut => {}
                Err(err) => panic!("failed to read from port: {err}"),
            }
        }

        self.backlog.pop_front()
    }
}
