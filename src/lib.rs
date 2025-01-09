pub mod api;
pub mod error;
pub mod game_loop;
pub mod models;

// Re-export common types
pub use error::GameError;
pub use models::{Game, GameRooms, Player};
