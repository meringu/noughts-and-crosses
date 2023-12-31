use core::fmt;

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameIDParseError {
    #[error("invalid length")]
    InvalidLength,
    #[error("must be alphanumeric")]
    InvalidCharacters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameID(String);

impl Default for GameID {
    fn default() -> Self {
        let r = rand::thread_rng();
        let s = r
            .sample_iter(rand::distributions::Alphanumeric)
            .take(Self::LENGTH)
            .map(char::from)
            .collect();

        Self(s)
    }
}

impl TryFrom<&str> for GameID {
    type Error = GameIDParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == Self::LENGTH {
            if value.chars().all(char::is_alphanumeric) {
                Ok(Self(value.to_string()))
            } else {
                Err(Self::Error::InvalidCharacters)
            }
        } else {
            Err(Self::Error::InvalidLength)
        }
    }
}

impl GameID {
    const LENGTH: usize = 8;

    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum PositionParseError {
    #[error("x must be less than 3")]
    XOutOfBounds,
    #[error("y must be less than 3")]
    YOutOfBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position(u8);

impl TryFrom<(usize, usize)> for Position {
    type Error = PositionParseError;

    fn try_from((x, y): (usize, usize)) -> Result<Self, Self::Error> {
        if x >= 3 {
            Err(Self::Error::XOutOfBounds)
        } else if y >= 3 {
            Err(Self::Error::YOutOfBounds)
        } else {
            // truncation is intentional. We only want to return specific bits.
            #[allow(clippy::cast_possible_truncation)]
            Ok(Self(((x as u8) << 6) + ((y as u8) << 4)))
        }
    }
}

impl Position {
    // x is the first 2 bits
    pub const fn x(&self) -> usize {
        (self.0 >> 6) as usize
    }
    // y is the second set of 2 bits
    pub const fn y(&self) -> usize {
        ((self.0 << 2) >> 6) as usize
    }
}
