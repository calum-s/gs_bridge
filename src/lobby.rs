use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::os::unix::net::UnixListener;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use std::thread;

type Socket = Recipient<WsMessage>;

#[derive(Default)]
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,     //self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>, //room id  to list of users id
}

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|user_id| {
                    self.send_message(&format!("{} disconnected.", &msg.id), user_id)
                });
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.rooms.remove(&msg.room_id);
                }
            }
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        self.rooms
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| {
                self.send_message(&format!("{} just joined!", msg.self_id), conn_id)
            });

        self.sessions.insert(msg.self_id, msg.addr);

        // Listen to Unix socket and send message to client on received message
        let socket_path = "/dev/tty.usbmodem111401".to_string();
        let (tx, rx) = channel();
        let lobby = Arc::new(Mutex::new(self));

        thread::spawn(move || {
            listen_to_unix_socket(socket_path, rx, || Arc::clone(&lobby));
        });

        // Handle messages received from the socket listener
        while let Ok(message) = rx.recv() {
            self.send_message(&message, &msg.self_id);
        }

        //self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}

fn listen_to_unix_socket(socket_path: String, rx: Receiver<String>, lobby: Lobby) {
    if let Ok(listener) = UnixListener::bind(&socket_path) {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 1024];
                if let Ok(bytes_read) = stream.read(&mut buffer) {
                    let message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                    let _ = rx.send(message);
                }
            }
        }
    } else {
        eprintln!("Failed to bind to Unix socket: {}", socket_path);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else if msg.msg.starts_with("<>") {
            // wait 250ms:
            //std::thread::sleep(std::time::Duration::from_millis(50));
            self.send_message(
                &("<<".to_string() + &msg.msg[2..]),
                &Uuid::parse_str(&msg.id.to_string()).unwrap(),
            );
        } else {
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .for_each(|client| self.send_message(&msg.msg, client));
        }
    }
}
