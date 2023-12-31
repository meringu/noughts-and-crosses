use serde::{Deserialize, Serialize};

use crate::{game_state::Position, Board};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerMessage {
    Error(String),
    WaitingForOpponentYouAreCrosses,
    GameUpdate(GameUpdate),
    OppositionRequestsRematch,
    GameEnded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameUpdate {
    pub board: Board,
    pub turn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientMessage {
    Move(Position),
    RequestRematch,
}
