use std::{net::SocketAddr, sync::Arc};

use log::{debug, error, info};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::common::{
    models::{Ball, ClientInput, ClientInputType, GameState},
    GameRooms,
};

fn validate_game_state(action: &ClientInputType, game_state: &GameState) -> bool {
    match action {
        ClientInputType::MovePaddle(_) => *game_state == GameState::Active,
        ClientInputType::JoinGame => *game_state == GameState::WaitingForPlayers,
        _ => true, // No validation needed for other actions
    }
}

pub async fn process_input(input: ClientInput, lobbies: Arc<Mutex<GameRooms>>, addr: SocketAddr) {
    let (game_id, player_id) = match (
        Uuid::parse_str(&input.game_id),
        Uuid::parse_str(&input.player_id),
    ) {
        (Ok(game_id), Ok(player_id)) => (game_id, player_id),
        _ => {
            info!("Invalid UUIDs of game_id or player_id");
            return;
        }
    };

    info!(
        "Processing input from player {} for game {}",
        player_id, game_id
    );

    let mut game_rooms = lobbies.lock().await;

    let game = match game_rooms.lobbies.get_mut(&game_id) {
        Some(game) => game,
        None => {
            error!("Game {} not found", game_id);
            return;
        }
    };

    if !validate_game_state(&input.action, &game.state) {
        debug!("Invalid action for game state");
        return;
    }

    let player = match game.get_player_mut(&player_id) {
        Some(player) => player,
        None => {
            error!("Player {} not found", player_id);
            return;
        }
    };

    match input.action {
        ClientInputType::JoinGame => {
            player.addr = Some(addr);
            player.ping_timestamp = Some(chrono::Utc::now());
            info!("game {}: {} ({}) joined", player.name, game_id, player_id);
        }
        ClientInputType::PlayerReady => {
            player.is_ready = !player.is_ready;
            if player.is_ready {
                info!("game {}: {} ({}) is ready", player.name, game_id, player_id);
            } else {
                info!(
                    "game {}: {} ({}) is not ready",
                    player.name, game_id, player_id
                );
            }

            if game.start_game().is_ok() {
                info!("game {}: started", game_id);
                game.ball = Some(Ball::new());
            }
        }
        ClientInputType::PauseGame => {
            if game.pause_game().is_ok() {
                info!("game {}: paused", game_id);
            }
        }
        ClientInputType::MovePaddle(direction) => {
            player.move_paddle(direction);
        }
        ClientInputType::Disconnect => {
            info!(
                "game {}: {} ({}) disconnected",
                player.name, game_id, player_id
            );
            game.remove_player(player_id);
        }
        ClientInputType::Ping => {
            debug!("Pong from player {}", player_id);
            player.ping_timestamp = Some(chrono::Utc::now());
        }
        _ => {
            error!("Unhandled action: {:?}", input.action);
        }
    }
}
