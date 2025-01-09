use std::{net::SocketAddr, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    models::{Ball, ClientInput, ClientInputType, Direction},
    GameRooms,
};

pub async fn process_input(input: ClientInput, lobbies: Arc<Mutex<GameRooms>>, addr: SocketAddr) {
    //println!("Handling incoming message");
    let game_id = match Uuid::parse_str(&input.game_id) {
        Ok(id) => id,
        Err(_) => return (),
    };

    let player_id = match Uuid::parse_str(&input.player_id) {
        Ok(id) => id,
        Err(_) => return (),
    };

    let mut game_rooms = lobbies.lock().await;

    let game = game_rooms.lobbies.get_mut(&game_id).unwrap();
    let player = game.get_player_mut(&player_id).unwrap();

    match input.action {
        ClientInputType::JoinGame => {
            player.addr = Some(addr);
            println!("game {}: new player joined", game.id);
        }
        ClientInputType::StartGame => {
            if game.start_game().is_ok() {
                println!("game {}: started", game.id);
                game.ball = Some(Ball::new());
            }
        }
        ClientInputType::PauseGame => {
            if game.pause_game().is_ok() {
                println!("game {}: paused", game.id);
            }
        }
        ClientInputType::MovePaddle(direction) => {
            let delta = match direction {
                Direction::Positive => player.paddle_delta,
                Direction::Negative => -player.paddle_delta,
            };
            player.paddle_position = (player.paddle_position + delta).clamp(-10.0, 10.0);
        }
        _ => {
            println!("Invalid action");
        }
    }
}
