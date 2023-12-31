use std::{rc::Rc, sync::Mutex};

use futures_util::StreamExt;
use worker::{
    async_trait, durable_object, js_sys, wasm_bindgen, wasm_bindgen_futures, worker_sys, Env,
    Request, Response, Result, State, WebSocketPair, WebsocketEvent,
};

use noughts_and_crosses_core::messages::ServerMessage;

use crate::{game_state::GameState, send_message};

#[durable_object]
struct Game {
    game_state: Rc<Mutex<GameState>>,

    // This durable object doesn't actually store any state. Re-connecting isn't allowed.
    // If the durable object restarts, the game will end.
    #[allow(dead_code)]
    state: State,
}

#[durable_object]
impl DurableObject for Game {
    fn new(state: State, _: Env) -> Self {
        Self {
            game_state: Rc::new(Mutex::new(GameState::default())),
            state,
        }
    }

    #[allow(clippy::unused_async)] // must be async to satisfy the async trait
    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        let WebSocketPair { client, server } = WebSocketPair::new()?;

        let session = Rc::new(server);
        let game_state = self.game_state.clone();

        session.accept()?;

        wasm_bindgen_futures::spawn_local(async move {
            {
                let mut game_state = game_state.lock().unwrap();
                if let Err(e) = game_state.join(&session) {
                    drop(game_state);
                    send_message(&ServerMessage::Error(e.to_string()), &session)
                        .expect("send error to client");
                    return;
                };
            }

            let events = session.events();
            if let Ok(mut stream) = events {
                while let Some(Ok(event)) = stream.next().await {
                    match event {
                        WebsocketEvent::Message(msg) => {
                            if let Some(bytes) = msg.bytes() {
                                let mut game_state = game_state.lock().unwrap();

                                if let Err(e) = game_state.handle_message(&session, &bytes) {
                                    send_message(&ServerMessage::Error(e.to_string()), &session)
                                        .expect("send error to client");
                                    return;
                                }
                            }
                        }
                        WebsocketEvent::Close(_) => {
                            let mut game_state = game_state.lock().unwrap();
                            game_state.player_left(&session).expect("closing game");
                        }
                    }
                }
            }
        });

        Response::from_websocket(client)
    }
}
