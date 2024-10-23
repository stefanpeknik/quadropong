use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use pong_server::{
    api::{create_game, get_game_by_id, get_games, join_game},
    models::{ClientInput, ClientInputType},
    GameRooms,
};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, time};
use uuid::Uuid;

async fn process_input(input: ClientInput, lobbies: Arc<Mutex<GameRooms>>, addr: SocketAddr) {
    let game_id = Uuid::parse_str(&input.game_id).unwrap();
    let player_id = Uuid::parse_str(&input.player_id).unwrap();

    let mut game_rooms = lobbies.lock().await;

    let game = game_rooms.lobbies.get_mut(&game_id).unwrap();
    let player = game.get_player_mut(&player_id).unwrap();

    match input.action {
        ClientInputType::JoinGame => {
            player.addr = Some(addr);
        }
        _ => {
            println!("Invalid action");
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a shared GameRooms instance
    let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);

    let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();
    let _ = socket.set_nonblocking(true);
    let socket = Arc::new(socket);

    // Clone for the receiver task
    let socket_recv = socket.clone();
    let game_rooms_recv = game_rooms.clone();

    let game_rooms_send = game_rooms.clone();

    // Spawn UDP receiver task
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            match socket_recv.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    // Try msgpack deserialization as fallback
                    if let Ok(input) = rmp_serde::from_slice::<ClientInput>(&buf[..size]) {
                        println!(
                            "Received msgpack input for game: {:?} from {}",
                            input.action, addr
                        );
                        process_input(input, game_rooms_recv.clone(), addr).await;
                    } else {
                        println!("Error deserializing input - size: {}", size);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, continue
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                Err(e) => eprintln!("Error receiving UDP packet: {}", e),
            }
        }
    });

    tokio::spawn(async move {
        // Game state broadcast loop
        let mut interval = time::interval(Duration::from_millis(1000 / 2));
        loop {
            interval.tick().await;
            println!("Sending the game state");
            let rooms = game_rooms_send.lock().await;
            for game in rooms.lobbies.values() {
                let serialized = rmp_serde::to_vec(&game).unwrap();

                for player in game.players.values() {
                    if let Some(addr) = player.addr {
                        if let Err(e) = socket.send_to(&serialized, addr) {
                            eprintln!("Error sending player state: {}", e);
                        }
                    }
                }
            }
        }
    });

    // Build the Axum app with routes
    let app = Router::new()
        .route("/game/:id", get(get_game_by_id)) // get game by id
        .route("/game", get(get_games)) // get list of all games
        .route("/game", post(create_game)) // create a new game
        .route("/game/:id/join", post(join_game)) // join a game
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
