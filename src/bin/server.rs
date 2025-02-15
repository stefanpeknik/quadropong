use chrono::Utc;
use log::{error, info};
use quadropong::common::{
    game_loop::process_input,
    models::{ClientInput, ClientInputWithAddr},
    GameRooms,
};
use std::{collections::VecDeque, env, net::UdpSocket, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

use quadropong::server::api::app;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug) // Set global log level
        .chain(std::io::stdout()) // Log to stdout
        .chain(fern::log_file(format!(
            "{}-quadropong-server.log",
            Utc::now().format("%Y-%m-%d-%H-%M-%S")
        ))?) // Log to file
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = setup_logger(); // Ignore logger failure

    // Create a shared GameRooms instance
    let game_rooms = Arc::new(Mutex::new(GameRooms::new()));

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let udp_port: u16 = env::var("UDP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(34254);

    let addr = format!("0.0.0.0:{}", port);

    let socket =
        UdpSocket::bind(format!("0.0.0.0:{}", udp_port)).expect("Failed to bind to UDP socket");
    let _ = socket.set_nonblocking(true);
    let socket = Arc::new(socket);

    // Clone for the receiver task
    let socket_recv = socket.clone();

    let game_rooms_send = game_rooms.clone();

    let message_queue: Arc<Mutex<VecDeque<ClientInputWithAddr>>> =
        Arc::new(Mutex::new(VecDeque::new()));

    // Spawn UDP receiver task
    let message_queue_recv = message_queue.clone();
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            match socket_recv.recv_from(&mut buf) {
                Ok((size, addr)) => match rmp_serde::from_slice::<ClientInput>(&buf[..size]) {
                    Ok(input) => {
                        let input = ClientInputWithAddr { addr, input };
                        message_queue_recv.lock().await.push_back(input);
                    }
                    Err(e) => {
                        error!("Failed to deserialize UDP packet: {}", e);
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::task::yield_now().await;
                }
                Err(e) => error!("Failed to receive UDP packet: {}", e),
            }
        }
    });

    let game_rooms_cleaner = game_rooms.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            info!("Running game room cleaner process");
            interval.tick().await;
            game_rooms_cleaner.lock().await.delete_games();
        }
    });

    let game_rooms_loop = game_rooms.clone();
    let message_queue_loop = message_queue.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(1000 / 60));
        loop {
            interval.tick().await;

            // Process all messages in the queue
            let mut queue = message_queue_loop.lock().await;
            while let Some(input) = queue.pop_front() {
                process_input(input.input, game_rooms_loop.clone(), input.addr).await;
            }

            let mut rooms = game_rooms_loop.lock().await;
            for game in rooms.lobbies.values_mut() {
                game.game_tick();
            }
        }
    });

    tokio::spawn(async move {
        // Game state broadcast loop
        let mut interval = time::interval(Duration::from_millis(1000 / 60));
        loop {
            interval.tick().await;

            let games = {
                let rooms = game_rooms_send.lock().await;
                rooms.lobbies.values().cloned().collect::<Vec<_>>()
            };

            // Broadcast the game state to all players
            for game in games {
                match game.to_network_bytes() {
                    Ok(serialized) => {
                        for player in game.players.values() {
                            if let Some(addr) = player.addr {
                                if let Err(e) = socket.send_to(&serialized, addr) {
                                    error!(
                                        "Failed to send game state to player {} on {}: {}",
                                        player.id, addr, e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize game state: {}", e);
                    }
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await;

    match listener {
        Ok(listener) => {
            info!("Listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app(game_rooms)).await.unwrap();
        }
        Err(e) => {
            error!("Failed to start listening: {}", e);
        }
    }
}
