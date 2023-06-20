// GS_Bridge
// Calum Scott

mod lobby;
mod ws;
use std::{error::Error, sync::Arc};

use lobby::Lobby;
mod messages;
mod start_connection;
use actix::{Actor, Addr};
use messages::Broadcast;
use start_connection::start_connection as start_connection_route;

use serialport;
use std::io::BufRead;
use std::io::BufReader;
use std::time::Duration;

use actix_web::{web::Data, App, HttpServer};
use tokio::{io::AsyncReadExt, try_join};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let lobby = Arc::new(Lobby::default().start()); // Create a lobby
    let lobby_ref1 = lobby.clone();

    let http_future = HttpServer::new(move || {
        App::new()
            .service(start_connection_route) // Register route
            .app_data(Data::new(lobby_ref1.clone())) // Register lobby with route
    })
    .bind("127.0.0.1:8080")?
    .run();

    try_join!(
        async {
            http_future.await.map_err(|e| {
                let e: Box<dyn Error> = e.into();
                e
            })
        },
        //socket_handler(lobby),
        exit_code()
    )
    .map(|_| ())
}

async fn exit_code() -> Result<(), Box<dyn Error>> {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    println!("CTRL+C received, shutting down");

    std::process::exit(1);
}

async fn socket_handler(lobby: Arc<Addr<Lobby>>) -> Result<(), Box<dyn Error>> {
    let path = "/dev/tty.usbmodem111401";

    let serial_port = serialport::new(path, 9600)
        .timeout(Duration::from_millis(2000))
        .open()
        .expect("Failed to open serial port");

    // let output = "This is a test.\n".as_bytes();
    // serial_port.write_all(output).expect("Write failed");
    // serial_port.flush().unwrap();

    let mut reader = BufReader::new(serial_port);
    let mut my_str = String::new();

    loop {
        my_str.clear();
        match reader.read_line(&mut my_str) {
            Ok(_) => {
                lobby
                    .clone()
                    .send(Broadcast {
                        msg: my_str.to_owned(),
                    })
                    .await?;
            }
            Err(err) => {
                print!("Error reading from port: {}", err);
                std::process::exit(1);
            }
        }
    }
}
