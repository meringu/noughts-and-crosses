#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

use serde::{Deserialize, Serialize};

pub mod game_state;
pub mod messages;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tile {
    #[default]
    Unplayed,
    Nought,
    Cross,
}

impl Tile {
    pub fn is_cross(&self) -> bool {
        *self == Self::Cross
    }
    pub fn is_nought(&self) -> bool {
        *self == Self::Nought
    }
    pub fn is_unplayed(&self) -> bool {
        *self == Self::Unplayed
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameSummary {
    #[default]
    InProgress,
    NoughtWin,
    CrossWin,
    Tie,
}

impl GameSummary {
    pub fn is_finished(&self) -> bool {
        self.ne(&Self::InProgress)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    pub tiles: [[Tile; 3]; 3],
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        for row in &mut self.tiles {
            for tile in row.iter_mut() {
                *tile = Tile::Unplayed;
            }
        }
    }

    pub fn summary(&self) -> GameSummary {
        // check all the rows
        if self.tiles.iter().any(|row| row.iter().all(Tile::is_cross)) {
            return GameSummary::CrossWin;
        }
        if self.tiles.iter().any(|row| row.iter().all(Tile::is_nought)) {
            return GameSummary::NoughtWin;
        }
        // check all the columns
        if (0..3).any(|col| self.tiles.iter().all(|row| row[col].is_cross())) {
            return GameSummary::CrossWin;
        }
        if (0..3).any(|col| self.tiles.iter().all(|row| row[col].is_nought())) {
            return GameSummary::NoughtWin;
        }
        // check diagonals
        if (0..3).all(|i| self.tiles[i][i].is_cross()) {
            return GameSummary::CrossWin;
        }
        if (0..3).all(|i| self.tiles[i][i].is_nought()) {
            return GameSummary::NoughtWin;
        }
        if (0..3).all(|i| self.tiles[i][2 - i].is_cross()) {
            return GameSummary::CrossWin;
        }
        if (0..3).all(|i| self.tiles[i][2 - i].is_nought()) {
            return GameSummary::NoughtWin;
        }
        // if there are no more moves
        if self
            .tiles
            .iter()
            .all(|row| row.iter().all(|t| !t.is_unplayed()))
        {
            return GameSummary::Tie;
        }

        GameSummary::InProgress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_win_row_0() {
        assert!(
            Board {
                tiles: [[Tile::Cross; 3], [Tile::Unplayed; 3], [Tile::Unplayed; 3],]
            }
            .summary()
                == GameSummary::CrossWin
        )
    }

    #[test]
    fn test_nought_win_row_2() {
        assert!(
            Board {
                tiles: [[Tile::Unplayed; 3], [Tile::Unplayed; 3], [Tile::Nought; 3],]
            }
            .summary()
                == GameSummary::NoughtWin
        )
    }

    #[test]
    fn test_cross_win_col_0() {
        assert!(
            Board {
                tiles: [
                    [Tile::Cross, Tile::Unplayed, Tile::Unplayed],
                    [Tile::Cross, Tile::Unplayed, Tile::Unplayed],
                    [Tile::Cross, Tile::Unplayed, Tile::Unplayed],
                ]
            }
            .summary()
                == GameSummary::CrossWin
        )
    }

    #[test]
    fn test_cross_win_diagonal() {
        assert!(
            Board {
                tiles: [
                    [Tile::Cross, Tile::Unplayed, Tile::Unplayed],
                    [Tile::Unplayed, Tile::Cross, Tile::Unplayed],
                    [Tile::Unplayed, Tile::Unplayed, Tile::Cross],
                ]
            }
            .summary()
                == GameSummary::CrossWin
        )
    }

    #[test]
    fn test_nought_win_reverse_diagonal() {
        assert!(
            Board {
                tiles: [
                    [Tile::Cross, Tile::Cross, Tile::Nought],
                    [Tile::Cross, Tile::Nought, Tile::Cross],
                    [Tile::Nought, Tile::Cross, Tile::Cross],
                ]
            }
            .summary()
                == GameSummary::NoughtWin
        )
    }

    #[test]
    fn test_empty_in_progress() {
        assert!(
            Board {
                tiles: [
                    [Tile::Unplayed; 3],
                    [Tile::Unplayed; 3],
                    [Tile::Unplayed; 3],
                ]
            }
            .summary()
                == GameSummary::InProgress
        )
    }

    #[test]
    fn test_full_tie() {
        assert!(
            Board {
                tiles: [
                    [Tile::Nought, Tile::Cross, Tile::Nought],
                    [Tile::Nought, Tile::Cross, Tile::Nought],
                    [Tile::Cross, Tile::Nought, Tile::Cross],
                ]
            }
            .summary()
                == GameSummary::Tie
        )
    }
}
