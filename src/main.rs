use clap::Parser;
use package_parser::{DataReceiver, Package, PackageV1, SpoofedData, END, MAGIC};
use rusqlite::Connection;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::spawn;

#[derive(Parser)]
struct Cli {
    database_path: String,
}

fn main() {
    let Cli { database_path } = Cli::parse();

    let conn = Connection::open(database_path).expect("database connection failed");

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

    for receiver in get_receivers() {
        let tx = tx.clone();
        spawn(move || read_from_stream(receiver, tx));
    }

    let mut statement = conn
        .prepare("INSERT INTO data (sensor, timestamp, value) VALUES (?, ?, ?)")
        .expect("failed to prepare statement");

    for package in rx {
        println!("package: {:?}", &package);
        statement
            .execute([package.sensor, package.timestamp, package.value])
            .expect("failed to execute statement");
    }
}

fn get_receivers() -> Vec<Box<dyn DataReceiver>> {
    vec![Box::new(SpoofedData::from(&[
        &MAGIC as &[u8],
        &1i32.to_le_bytes(),
        &12i32.to_le_bytes(),
        &42i32.to_le_bytes(),
        &17i32.to_le_bytes(),
        &7i32.to_le_bytes(),
        &END,
    ]))]
}

fn read_from_stream(mut receiver: Box<dyn DataReceiver>, tx: Sender<PackageV1>) {
    let mut buffer = Vec::new();
    while let Some(next) = receiver.get_next_byte() {
        buffer.push(next);
        if let Some(package) = Package::try_from_buffer(&buffer) {
            println!("Got package");
            if let Some(package_v1) = PackageV1::try_from(package) {
                println!("Got package v1");
                tx.send(package_v1)
                    .expect("failed to send package via channel");
            }
        }
    }
}
