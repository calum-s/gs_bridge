// GS_Bridge
// Calum Scott

mod lobby;
mod ws;
use lobby::Lobby;
mod messages;
mod start_connection;
use actix::Actor;
use start_connection::start_connection as start_connection_route;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chat_server = Lobby::default().start(); // Create a lobby

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route) // Register route
            .data(chat_server.clone()) // Register lobby with route
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
