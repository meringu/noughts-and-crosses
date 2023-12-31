use std::rc::Rc;

use rand::Rng;
use worker::{Error, Result, WebSocket};

use noughts_and_crosses_core::{
    messages::{ClientMessage, GameUpdate, ServerMessage},
    Board, GameSummary, Tile,
};

use crate::send_message;

#[derive(Debug, Clone, Default)]
pub struct GameState {
    board: Board,
    crosses_session: Option<Rc<WebSocket>>,
    noughts_session: Option<Rc<WebSocket>>,

    crosses_turn: bool,

    crosses_requests_rematch: bool,
    noughts_requests_rematch: bool,
}

impl GameState {
    /// request to join a game
    pub fn join(&mut self, session: &Rc<WebSocket>) -> Result<()> {
        // First player to join is crosses
        if self.crosses_session.is_none() {
            self.crosses_session = Some(session.clone());
            send_message(&ServerMessage::WaitingForOpponentYouAreCrosses, session)?;
            return Ok(());
        }

        // Second player joining starts the game
        if self.noughts_session.is_none() {
            self.noughts_session = Some(session.clone());
            self.new_game()?;
            return Ok(());
        }

        Err(Error::RustError("this game is full".to_string()))
    }

    /// clears the previous game and flips a coin to see who starts.
    pub fn new_game(&mut self) -> Result<()> {
        self.board.clear();
        self.crosses_requests_rematch = false;
        self.noughts_requests_rematch = false;

        // flip a coin to see who starts
        let mut r = rand::thread_rng();
        self.crosses_turn = r.gen();

        // send opening board state to both players
        self.notify()?;

        Ok(())
    }

    /// called when a player leaves. The opposition is notified and the game ended.
    /// if state were stored, this would be a good place to remove it.
    pub fn player_left(&mut self, session: &Rc<WebSocket>) -> Result<()> {
        // check the session is actually a player
        let mut is_player = false;
        if let Some(crosses_session) = &self.crosses_session {
            if crosses_session.eq(session) {
                is_player = true;
            }
        }
        if let Some(noughts_session) = &self.noughts_session {
            if noughts_session.eq(session) {
                is_player = true;
            }
        }

        if !is_player {
            // The connection could be someone quickly connecting and disconnecting from the game.
            return Ok(());
        }

        // If both players are still connected, send the other player a close message and close the connection.
        if let Some(crosses_session) = &self.crosses_session {
            if let Some(noughts_session) = &self.noughts_session {
                if crosses_session == session {
                    send_message(&ServerMessage::GameEnded, noughts_session)?;
                    // TODO: return from crosses session handler
                }
                if noughts_session == session {
                    send_message(&ServerMessage::GameEnded, crosses_session)?;
                    // TODO: return from noughts session handler
                }
            }
        }

        // clear the sessions to this game ID could be re-used.
        self.crosses_session = None;
        self.noughts_session = None;

        Ok(())
    }

    /// notifies the sessions of the current state of play
    /// if reconnects and persistence were to be supported, this should also save the state to storage
    fn notify(&mut self) -> Result<()> {
        if let Some(session) = &self.crosses_session {
            send_message(
                &ServerMessage::GameUpdate(GameUpdate {
                    board: self.board,
                    turn: self.crosses_turn,
                }),
                session,
            )?;
        }

        if let Some(session) = &self.noughts_session {
            send_message(
                &ServerMessage::GameUpdate(GameUpdate {
                    board: self.board,
                    turn: !self.crosses_turn,
                }),
                session,
            )?;
        }

        Ok(())
    }

    /// handles a client message
    pub fn handle_message(&mut self, session: &Rc<WebSocket>, bytes: &[u8]) -> Result<()> {
        let message: ClientMessage = bincode::deserialize(bytes)
            .map_err(|_| Error::RustError("invalid message from client".to_string()))?;

        let crosses_session = self
            .crosses_session
            .clone()
            .ok_or(Error::RustError("this game is full".to_string()))?;
        let noughts_session = self
            .noughts_session
            .clone()
            .ok_or(Error::RustError("this game is full".to_string()))?;

        match message {
            ClientMessage::Move(pos) => {
                let cross_requesting = crosses_session.eq(session);

                match self.board.summary() {
                    GameSummary::InProgress => {
                        if self.crosses_turn != cross_requesting {
                            return Err(Error::RustError(
                                "it is not your turn to move".to_string(),
                            ));
                        }

                        if self.board.tiles[pos.x()][pos.y()] == Tile::Unplayed {
                            self.board.tiles[pos.x()][pos.y()] = if cross_requesting {
                                Tile::Cross
                            } else {
                                Tile::Nought
                            };
                        } else {
                            return Err(Error::RustError("invalid move".to_string()));
                        }
                    }
                    _ => {
                        return Err(Error::RustError("the game is not in progress".to_string()));
                    }
                }

                // switch turns
                self.crosses_turn = !self.crosses_turn;

                self.notify()?;
            }
            ClientMessage::RequestRematch => {
                // if this is the first time requesting rematch, save the client and ask the opponent
                // for a rematch if they haven't asked already themselves
                if crosses_session.eq(session) {
                    if !self.crosses_requests_rematch {
                        self.crosses_requests_rematch = true;
                        if !self.noughts_requests_rematch {
                            send_message(
                                &ServerMessage::OppositionRequestsRematch,
                                &noughts_session,
                            )?;
                        }
                    }
                } else if !self.noughts_requests_rematch {
                    self.noughts_requests_rematch = true;
                    if !self.crosses_requests_rematch {
                        send_message(&ServerMessage::OppositionRequestsRematch, &crosses_session)?;
                    }
                }

                if self.crosses_requests_rematch && self.noughts_requests_rematch {
                    // both players have requested a rematch
                    self.new_game()?;
                }
            }
        }

        Ok(())
    }
}
