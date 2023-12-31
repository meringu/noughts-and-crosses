#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
// worker-rs results are not send
#![allow(clippy::future_not_send)]

use js_sys::Uint8Array;
use worker::{
    event, js_sys, Env, Error, Request, Response, Result, RouteContext, Router, WebSocket,
};

use noughts_and_crosses_core::messages::ServerMessage;

mod game;
mod game_state;

fn index(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::from_json(&"Hello, World!")
}

/// sends a server message via a websocket
fn send_message(message: &ServerMessage, session: &WebSocket) -> Result<()> {
    let bytes = bincode::serialize(&message).map_err(|e| Error::RustError(e.to_string()))?;

    // TODO: we should be able to send the bytes directly with session.send_with_bytes
    // https://github.com/cloudflare/workers-rs/issues/379
    let uint8_array = Uint8Array::from(bytes.as_slice());
    Ok(session
        .as_ref()
        .send_with_array_buffer(&uint8_array.buffer())?)
}

/// extracts the game id from the header and forwards the request to the durable object
async fn websocket(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let game_id = ctx.param("id").unwrap();
    let namespace = ctx.durable_object("GAME")?;
    let stub = namespace.id_from_name(game_id)?.get_stub()?;
    stub.fetch_with_request(req).await
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    Router::new()
        .get("/", index)
        .on_async("/game/:id", websocket)
        .run(req, env)
        .await
}
