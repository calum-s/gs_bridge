use crate::messages::{Connect, Disconnect, WsMessage, Broadcast};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap};
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

#[derive(Default)]
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,     //self id to self
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.self_id, msg.addr);
    }
}

impl Handler<Broadcast> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, _ctx: &mut Self::Context) -> Self::Result {
        for (_, socket) in &self.sessions {
            let _ = socket.do_send(WsMessage(msg.msg.clone()));
        }
    }
}
