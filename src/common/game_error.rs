use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Game is full")]
    GameFull,
    #[error("Game not found")]
    GameNotFound,
    #[error("Player not found")]
    PlayerNotFound,
    #[error("Invalid game state transition")]
    InvalidStateTransition,
    #[error("Players are not ready")]
    PlayersNotReady,
}
