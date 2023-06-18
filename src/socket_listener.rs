use actix::prelude::*;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use uuid::Uuid;

pub struct SocketListener {
    lobby: Addr<Lobby>,
}

impl Actor for SocketListener {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start listening to the Unix socket
        let socket_path = "/path/to/socket.sock".to_string();
        if let Ok(listener) = UnixListener::bind(&socket_path) {
            let lobby = self.lobby.clone();
            ctx.spawn(async move {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let mut buffer = [0; 1024];
                        if let Ok(bytes_read) = stream.read(&mut buffer) {
                            let message =
                                String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

                            // Handle the received message in the Lobby actor
                            lobby
                                .send(HandleSocketMessage {
                                    message,
                                    self_id: Uuid::new_v4(),
                                })
                                .await
                                .unwrap();
                        }
                    }
                }
            });
        } else {
            eprintln!("Failed to bind to Unix socket: {}", socket_path);
        }
    }
}

impl Handler<HandleSocketMessage> for SocketListener {
    type Result = ();

    fn handle(&mut self, msg: HandleSocketMessage, _: &mut Context<Self>) -> Self::Result {
        // Handle the received socket message in the Lobby actor
        // Access the Lobby actor's state using `self.lobby` and process the message

        // Example:
        // self.lobby.send_message(&msg.message, &msg.self_id);
    }
}

pub struct HandleSocketMessage {
    pub message: String,
    pub self_id: Uuid,
}

pub fn start_socket_listener(lobby: Addr<Lobby>) {
    let _ = SocketListener::start(lobby);
}
