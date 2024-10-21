use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    result,
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::{uuid, Uuid};

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
}

#[derive(Debug, Serialize)]
struct ApiError {
    status: u16,
    message: Option<String>,
}

struct GameRooms {
    lobbies: HashMap<Uuid, Game>,
}

impl GameRooms {
    fn new() -> Self {
        Self {
            lobbies: HashMap::new(),
        }
    }

    fn create_game(&mut self) -> Uuid {
        let game = Game::new();
        let game_id = game.id.clone();
        self.lobbies.insert(game_id, game);

        game_id
    }

    fn find_lobby_mut(&mut self, id: Uuid) -> Option<&mut Game> {
        self.lobbies.get_mut(&id)
    }

    fn find_lobby(&mut self, id: Uuid) -> Option<&Game> {
        self.lobbies.get(&id)
    }
}

#[derive(Debug, Serialize, Clone)]
enum GameState {
    WaitingForPlayers,
    Active,
    Paused,
    Finished,
}

#[derive(Serialize, Clone)]
struct Player {
    id: Uuid,
    name: String,
    score: u32,
}

impl Player {
    fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name,
            score: 0,
        }
    }

    fn increment_score(&mut self) {
        self.score += 1;
    }
}

#[derive(Serialize, Clone)]
struct Game {
    id: Uuid,
    players: HashMap<Uuid, Player>,
    state: GameState,
    max_players: usize,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Game ID: {}", self.id)?;
        writeln!(f, "State: {:?}", self.state)?;
        writeln!(f, "Players ({}):", self.players.len())?;
        for (_, player) in &self.players {
            writeln!(
                f,
                "  - {} (ID: {}), Score: {}",
                player.name, player.id, player.score
            )?;
        }
        Ok(())
    }
}

impl Game {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            players: HashMap::new(),
            state: GameState::WaitingForPlayers,
            max_players: 4,
            created_at: chrono::Utc::now(),
        }
    }

    fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        if self.players.len() >= self.max_players {
            return Err(GameError::GameFull);
        }
        self.players.insert(player.id, player);
        Ok(())
    }

    fn remove_player(&mut self, id: Uuid) {
        self.players.remove(&id);
    }

    fn set_game_state(&mut self, state: GameState) {
        self.state = state;
    }

    fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    fn get_player(&self, id: &Uuid) -> Option<&Player> {
        self.players.get(id)
    }

    fn get_player_mut(&mut self, id: &Uuid) -> Option<&mut Player> {
        self.players.get_mut(id)
    }
}

struct GameSummary {
    id: Uuid,
    players_count: usize,
    state: GameState,
}

// Endpoint to create a new game
async fn create_game(State(app_state): State<Arc<Mutex<GameRooms>>>) -> (StatusCode, Json<Game>) {
    let mut game_rooms = app_state.lock().await;

    let new_game_id = game_rooms.create_game();
    let game = game_rooms.find_lobby(new_game_id);

    match game {
        Some(game) => (StatusCode::OK, Json(game.clone())),
        None => (StatusCode::INTERNAL_SERVER_ERROR, Json(Game::new())),
    }
}

async fn get_games(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
) -> (StatusCode, Json<Vec<Game>>) {
    let game_rooms = app_state.lock().await;

    let result: Vec<Game> = game_rooms.lobbies.values().cloned().collect();

    (StatusCode::OK, Json(result))
}

async fn get_game_by_id(
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

#[derive(Deserialize)]
struct JoinGameRequest {
    username: Option<String>,
}

async fn join_game(
    State(app_state): State<Arc<Mutex<GameRooms>>>,
    Path(game_id): Path<String>,
    Json(payload): Json<JoinGameRequest>,
) -> Result<Json<Player>, StatusCode> {
    let game_uuid = Uuid::parse_str(&game_id).map_err(|e| StatusCode::BAD_REQUEST)?;

    let mut game_rooms = app_state.lock().await;

    let game = game_rooms
        .lobbies
        .get_mut(&game_uuid)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Generate player name based on request or player count
    let player_name = payload.username.unwrap_or_else(|| {
        let player_number = game.players.len() + 1;
        format!("player_{}", player_number)
    });

    let player = Player::new(player_name);
    let player_copy = player.clone();

    game.add_player(player)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|_| Json(player_copy))
}

#[tokio::main]
async fn main() {
    // Create a shared GameRooms instance
    let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);

    // Build the Axum app with routes
    let app = Router::new()
        .route("/game/:id", get(get_game_by_id))
        .route("/game", get(get_games))
        .route("/game", post(create_game))
        .route("/game/:id/join", post(join_game))
        .with_state(game_rooms);

    let listener = tokio::net::TcpListener::bind(addr).await;

    match listener {
        Ok(listener) => {
            info!("Starting server on port {}", port);
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
