use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::common::{models::GameState, Game, GameRooms, JoinGameRequest, Player};

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

    if game.state != GameState::WaitingForPlayers {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Generate player name based on request or player count
    let player_name = match payload.username {
        Some(name) if !name.is_empty() => name,
        _ => {
            let player_number = game.players.len() + 1;
            format!("player_{}", player_number)
        }
    };

    let player_positions = game.assign_position();

    let mut player = Player::new(player_name, false);

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

pub async fn add_bot(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
    Path(game_id): Path<String>,
) -> Result<Json<Player>, StatusCode> {
    let game_uuid = Uuid::parse_str(&game_id).map_err(|_e| StatusCode::BAD_REQUEST)?;

    let mut game_rooms = app_state.lock().await;

    let game = game_rooms
        .lobbies
        .get_mut(&game_uuid)
        .ok_or(StatusCode::NOT_FOUND)?;

    if (game.players.len() + 1) > 4 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let player_name = format!("bot_{}", game.players.len() + 1);

    let mut player = Player::new(player_name, true);

    let player_positions = game.assign_position();

    if let Some(position) = player_positions {
        player.position = Some(position);
    }

    let player_copy = player.clone();

    game.add_player(player)
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|_| Json(player_copy))
}

pub async fn restart_game(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
    Path(game_id): Path<String>,
    Json(payload): Json<JoinGameRequest>,
) -> Result<Json<Player>, StatusCode> {
    let game_uuid = Uuid::parse_str(&game_id).map_err(|_e| StatusCode::BAD_REQUEST)?;

    let mut game_rooms = app_state.lock().await;

    let game = game_rooms.lobbies.get_mut(&game_uuid);

    let game = match game {
        Some(game) => game,
        None => return Err(StatusCode::NOT_FOUND),
    };

    if game.state == GameState::Finished {
        game.set_game_state(GameState::WaitingForPlayers);
        game.started_at = None;
        game.finished_at = None;
        game.players.clear();
    }

    if game.state != GameState::WaitingForPlayers {
        return Err(StatusCode::BAD_REQUEST);
    }

    let player_name = match payload.username {
        Some(name) if !name.is_empty() => name,
        _ => {
            let player_number = game.players.len() + 1;
            format!("player_{}", player_number)
        }
    };

    let player_positions = game.assign_position();

    let mut player = Player::new(player_name, false);

    if let Some(position) = player_positions {
        player.position = Some(position);
    }

    let player_copy = player.clone();

    game.add_player(player)
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|_| Json(player_copy))
}

pub async fn remove_bot(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
    Path(game_id): Path<String>,
) -> Result<(), StatusCode> {
    let game_uuid = Uuid::parse_str(&game_id).map_err(|_e| StatusCode::BAD_REQUEST)?;

    let mut game_rooms = app_state.lock().await;

    let game = game_rooms
        .lobbies
        .get_mut(&game_uuid)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(bot) = game.players.values().find(|p| p.is_ai) {
        game.remove_player(bot.id);
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    return Ok(());
}

// Build the Axum app with routes
pub fn app(game_rooms: Arc<Mutex<GameRooms>>) -> Router {
    Router::new()
        .route("/game/:id", get(get_game_by_id)) // get game by id
        .route("/game", get(get_games)) // get list of all games
        .route("/game", post(create_game)) // create a new game
        .route("/game/:id/join", post(join_game)) // join a game
        .route("/game/:id/add_bot", post(add_bot)) // add a bot to a game
        .route("/game/:id/play_again", post(restart_game)) // add a bot to a game
        .route("/game/:id/remove_bot", post(remove_bot)) // remove a bot from a game
        .with_state(game_rooms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_game() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/game")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Game = serde_json::from_slice(&body).unwrap();

        assert_eq!(game_rooms.lock().await.lobbies.len(), 1);
        assert_eq!(game_rooms.lock().await.lobbies[&body.id], body);
    }

    #[tokio::test]
    async fn test_get_games() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/game")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Vec<Game> = serde_json::from_slice(&body).unwrap();

        assert!(body.is_empty());

        game_rooms.lock().await.create_game();

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/game")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Vec<Game> = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.len(), 1);
        assert_eq!(
            body[0],
            game_rooms
                .lock()
                .await
                .lobbies
                .values()
                .next()
                .unwrap()
                .clone()
        );
    }

    #[tokio::test]
    async fn test_get_game_by_id() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let game_id = game_rooms.lock().await.create_game();

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/game/{}", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Game = serde_json::from_slice(&body).unwrap();

        assert_eq!(body, game_rooms.lock().await.lobbies[&game_id]);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/game/invalid_id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let game_id = game_rooms.lock().await.create_game();
        let random_id = Uuid::new_v4();
        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/game/{}", random_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(game_id != random_id);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_join_game() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let game_id = game_rooms.lock().await.create_game();

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/join", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "test");
        assert_eq!(body.is_ai, false);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/join", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "player_2"); // default name because of empty username
        assert_eq!(body.is_ai, false);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/join", game_id))
                    .header("content-type", "application/json")
                    .body(json!({}).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "player_3"); // default name because of empty username
        assert_eq!(body.is_ai, false);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/join", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "test");
        assert_eq!(body.is_ai, false);

        let random_game_id = Uuid::new_v4();
        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/join", random_game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_add_bot() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let game_id = game_rooms.lock().await.create_game();

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/add_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "bot_1");
        assert_eq!(body.is_ai, true);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/add_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "bot_2");
        assert_eq!(body.is_ai, true);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/add_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "bot_3");
        assert_eq!(body.is_ai, true);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/add_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "bot_4");
        assert_eq!(body.is_ai, true);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/add_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let random_game_id = Uuid::new_v4();
        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/add_bot", random_game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_remove_bot() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let game_id = game_rooms.lock().await.create_game();

        let bot = Player::new("bot".to_string(), true);
        game_rooms
            .lock()
            .await
            .lobbies
            .get_mut(&game_id)
            .unwrap()
            .add_player(bot)
            .unwrap();

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/remove_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/remove_bot", game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let random_game_id = Uuid::new_v4();
        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/remove_bot", random_game_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_restart_game() {
        let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

        let game_id = game_rooms.lock().await.create_game();

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/play_again", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "test");
        assert_eq!(body.is_ai, false);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/play_again", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "player_2"); // default name because of empty username
        assert_eq!(body.is_ai, false);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/play_again", game_id))
                    .header("content-type", "application/json")
                    .body(json!({}).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "player_3"); // default name because of empty username
        assert_eq!(body.is_ai, false);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/play_again", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Player = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.name, "test");
        assert_eq!(body.is_ai, false);

        let random_game_id = Uuid::new_v4();
        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/play_again", random_game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let _game = game_rooms.lock().await.lobbies.get(&game_id).unwrap();
        game_rooms.lock().await.lobbies.remove(&game_id);

        let response = app(game_rooms.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/game/{}/play_again", game_id))
                    .header("content-type", "application/json")
                    .body(json!({ "username": "test" }).to_string().to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
