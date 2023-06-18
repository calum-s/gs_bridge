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

use actix_web::{App, HttpServer, web::Data};
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
        socket_handler(lobby)
    ).map(|_| ())
}

async fn socket_handler(lobby: Arc<Addr<Lobby>>) -> Result<(), Box<dyn Error>> {
    let mut listener = tokio::net::UnixStream::connect("/tmp/echo.sock").await?;
    loop {
        let mut buf = [0; 1024];
        let n = listener.read(&mut buf).await?;
        println!("read {} bytes", n);
        lobby
            .clone()
            .send(Broadcast {
                msg: String::from_utf8_lossy(&buf[..n]).into_owned(),
            })
            .await?;
    }
}
