use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::common::{Game, GameRooms, JoinGameRequest, Player};

pub async fn join_game(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
    Path(game_id): Path<String>,
    Json(payload): Json<JoinGameRequest>,
) -> Result<Json<Player>, StatusCode> {
    let game_uuid = Uuid::parse_str(&game_id).map_err(|_e| StatusCode::BAD_REQUEST)?;

    let mut game_rooms = app_state.lock().await;

    let game = game_rooms
        .lobbies
        .get_mut(&game_uuid)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Generate player name based on request or player count
    let player_name = match payload.username {
        Some(name) if !name.is_empty() => name,
        _ => {
            let player_number = game.players.len() + 1;
            format!("player_{}", player_number)
        }
    };

    let player_positions = game.assign_position();

    let mut player = Player::new(player_name);

    if let Some(position) = player_positions {
        player.position = Some(position);
    }

    let player_copy = player.clone();

    game.add_player(player)
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|_| Json(player_copy))
}

// Endpoint to create a new game
pub async fn create_game(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
) -> (StatusCode, Json<Game>) {
    let mut game_rooms = app_state.lock().await;

    let new_game_id = game_rooms.create_game();
    let game = game_rooms.find_lobby(new_game_id);

    match game {
        Some(game) => (StatusCode::OK, Json(game.clone())),
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(Game::new())),
    }
}

pub async fn get_games(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
) -> (StatusCode, Json<Vec<Game>>) {
    let game_rooms = app_state.lock().await;

    let result: Vec<Game> = game_rooms.lobbies.values().cloned().collect();

    (StatusCode::OK, Json(result))
}

pub async fn get_game_by_id(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
    Path(game_id): Path<String>,
) -> Result<Json<Game>, StatusCode> {
    let game_uuid = Uuid::parse_str(&game_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let game_rooms = app_state.lock().await;

    game_rooms
        .lobbies
        .get(&game_uuid)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
