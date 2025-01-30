use std::{net::SocketAddr, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::common::{
    models::{Ball, ClientInput, ClientInputType, Direction, GameState},
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
        _ => return,
    };

    let mut game_rooms = lobbies.lock().await;

    let game = match game_rooms.lobbies.get_mut(&game_id) {
        Some(game) => game,
        None => return,
    };

    if !validate_game_state(&input.action, &game.state) {
        return;
    }

    let player = match game.get_player_mut(&player_id) {
        Some(player) => player,
        None => return,
    };

    match input.action {
        ClientInputType::JoinGame => {
            player.addr = Some(addr);
            println!("game {}: new player joined", game_id);
        }
        ClientInputType::PlayerReady => {
            player.is_ready = true;

            if game.start_game().is_ok() {
                println!("game {}: started", game_id);
                game.ball = Some(Ball::new());
            }
        }
        ClientInputType::PauseGame => {
            if game.pause_game().is_ok() {
                println!("game {}: paused", game_id);
            }
        }
        ClientInputType::MovePaddle(direction) => {
            let delta = match direction {
                Direction::Positive => player.paddle_delta,
                Direction::Negative => -player.paddle_delta,
            };
            player.paddle_position = (player.paddle_position + delta).clamp(
                0.0 + (player.paddle_width / 2.0),
                10.0 - (player.paddle_width / 2.0),
            );
        }
        ClientInputType::Disconnect => {
            println!("game {}: player {} disconnected", game_id, player_id);
            game.remove_player(player_id);
        }
        _ => {
            println!("Invalid action");
        }
    }
}
