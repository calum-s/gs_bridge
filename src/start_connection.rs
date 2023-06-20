use std::sync::Arc;

use crate::lobby::Lobby;
use crate::ws::WsConn;
use actix::Addr;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

#[get("/")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Arc<Addr<Lobby>>>,
) -> Result<HttpResponse, Error> {
    println!("New connection");
    let ws = WsConn::new(srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
