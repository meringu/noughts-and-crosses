use bevy::prelude::*;
use noughts_and_crosses_core::{messages::ServerMessage, Board};

use crate::client::ServerEvent;

pub struct Plugin;

const WIN_MESSAGE: &str = "You won! Click to request a rematch.";
const LOST_MESSAGE: &str = "You lost. Click to request a rematch.";

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, update);
    }
}

#[derive(Resource, Default)]
#[allow(clippy::struct_excessive_bools)] // todo
pub struct GameState {
    pub board: Board,
    pub turn: bool,
    pub is_crosses: bool,
    pub blocking_message: Option<String>,
    pub rematch_requested: bool,
    pub ended: bool,
}

fn setup(mut commands: Commands) {
    // start the game in a loading state. This will transition once the websocket is established and the first message is received
    let game_state = GameState {
        blocking_message: Some("Loading...".to_string()),
        ..default()
    };
    commands.insert_resource(game_state);
}

fn update(mut ev_server: EventReader<ServerEvent>, mut game_state: ResMut<GameState>) {
    for ev in ev_server.read() {
        match &ev.message {
            ServerMessage::Error(e) => {
                game_state.blocking_message = Some(e.to_string());
                game_state.ended = true; // lock the game up as if the opponent left
            }
            ServerMessage::WaitingForOpponentYouAreCrosses => {
                game_state.is_crosses = true;
                game_state.blocking_message =
                    Some("Waiting for Opponent. Send the URL to a friend".to_string());
            }
            ServerMessage::GameUpdate(update) => {
                game_state.board = update.board;
                game_state.turn = update.turn;
                game_state.rematch_requested = false;
                match update.board.summary() {
                    noughts_and_crosses_core::GameSummary::InProgress => {
                        game_state.blocking_message = if update.turn {
                            None
                        } else {
                            Some("It is your opponents turn".to_string())
                        }
                    }
                    noughts_and_crosses_core::GameSummary::NoughtWin => {
                        game_state.blocking_message = Some(
                            if game_state.is_crosses {
                                LOST_MESSAGE
                            } else {
                                WIN_MESSAGE
                            }
                            .to_string(),
                        );
                    }
                    noughts_and_crosses_core::GameSummary::CrossWin => {
                        game_state.blocking_message = Some(
                            if game_state.is_crosses {
                                WIN_MESSAGE
                            } else {
                                LOST_MESSAGE
                            }
                            .to_string(),
                        );
                    }
                    noughts_and_crosses_core::GameSummary::Tie => {
                        game_state.blocking_message = Some("It is a draw".to_string());
                    }
                }
            }
            ServerMessage::OppositionRequestsRematch => {
                game_state.blocking_message =
                    Some("Your opponent has requested a rematch. Click to oblige.".to_string());
            }
            ServerMessage::GameEnded => {
                game_state.blocking_message = Some("Your opponent has left".to_string());
                game_state.ended = true;

                // TODO, option to start new game
            }
        }
    }
}
