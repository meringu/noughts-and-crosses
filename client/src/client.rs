use bevy::prelude::*;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::js_sys::Uint8Array;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use noughts_and_crosses_core::{
    game_state::GameID,
    messages::{ClientMessage, ServerMessage},
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update)
            .add_event::<ServerEvent>()
            .add_event::<ClientEvent>();
    }
}

#[derive(Event)]
pub struct ServerEvent {
    pub message: ServerMessage,
}
impl From<ServerMessage> for ServerEvent {
    fn from(message: ServerMessage) -> Self {
        Self { message }
    }
}

#[derive(Event)]
#[allow(clippy::module_name_repetitions)]
pub struct ClientEvent {
    message: ClientMessage,
}
impl From<ClientMessage> for ClientEvent {
    fn from(message: ClientMessage) -> Self {
        Self { message }
    }
}

#[derive(Resource)]
struct MessageReceiver(UnboundedReceiver<ServerMessage>);

#[derive(Resource)]
struct MessageSender(UnboundedSender<ClientMessage>);

fn setup(mut commands: Commands) {
    // use channels to connect bevy events to background tasks to communicate to the server
    let (server_sender, server_receiver) = mpsc::unbounded_channel();
    commands.insert_resource(MessageReceiver(server_receiver));
    let (client_sender, mut client_receiver) = mpsc::unbounded_channel();
    commands.insert_resource(MessageSender(client_sender));

    // get or set the game id
    let window = web_sys::window().unwrap();
    let game_id = window.location().pathname().map_or_else(
        |_| GameID::new(),
        |pathname| {
            GameID::try_from(pathname.strip_prefix('/').unwrap_or(""))
                .map_or_else(|_| GameID::new(), |game_id| game_id)
        },
    );

    // update the URL to the current game
    if let Ok(pathname) = window.location().pathname() {
        let desired = format!("/{game_id}");
        if pathname != desired {
            window
                .history()
                .unwrap()
                .push_state_with_url(&JsValue::NULL, "", Some(&desired))
                .unwrap();
        }
    }

    let ws = WebSocket::new(&format!("ws://127.0.0.1:8787/game/{game_id}"))
        .expect("failed to open connection to server");
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    // when a message is received, pass it to the server_sender channel
    let sen = server_sender.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(abuf) = e.data().dyn_into() {
            let bytes = Uint8Array::new(&abuf).to_vec();
            let message: ServerMessage = bincode::deserialize(&bytes).unwrap();
            sen.send(message).unwrap();
        }
    }) as Box<dyn FnMut(MessageEvent)>);

    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    // on error callback just prints the error to the console
    let onerror_callback = Closure::wrap(Box::new(move |_: ErrorEvent| {
        server_sender
            .send(ServerMessage::Error("Connection error.".to_string()))
            .unwrap();
    }) as Box<dyn FnMut(_)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    ws.set_onclose(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // background task to send messages to the server from the client receiver
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(message) = client_receiver.recv().await {
            let bytes = bincode::serialize(&message).unwrap();
            ws.send_with_u8_array(&bytes).unwrap();
        }
    });
}

fn update(
    mut receiver: ResMut<MessageReceiver>,
    sender_channel: Res<MessageSender>,

    mut ev_server: EventWriter<ServerEvent>,
    mut ev_client: EventReader<ClientEvent>,
) {
    // read all client messages and put them on the sender channel for the background task to send
    for event in ev_client.read() {
        sender_channel.0.send(event.message.clone()).unwrap();
    }

    // read all server messages on the receiver channel and add them to the server event stream
    while let Ok(message) = receiver.0.try_recv() {
        ev_server.send(message.into());
    }
}
