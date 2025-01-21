use crate::client::net::udp::UdpClient;
use crate::common::models::{ClientInput, ClientInputType, Direction, GameDto, PlayerDto};

use super::menu::Menu;
use super::traits::{Render, State, Update};
use super::utils::render::{calculate_game_area, render_ball, render_players};

use crossterm::event::KeyCode;
use ratatui::widgets::Block;
use ratatui::Frame;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

enum Options {
    Active,
    Paused,
    Finished,
}

pub struct GameBoard {
    game: Arc<Mutex<GameDto>>,
    our_player_id: Uuid,
    receive_updates: Arc<AtomicBool>,
    receive_update_handle: tokio::task::JoinHandle<()>,
    udp_client: Arc<UdpClient>,
}

impl GameBoard {
    pub fn new(game: GameDto, our_player_id: Uuid, udp_client: Arc<UdpClient>) -> Self {
        let game = Arc::new(Mutex::new(game));
        let receive_updates = Arc::new(AtomicBool::new(true));
        let receive_updates_clone = Arc::clone(&receive_updates);
        let game_clone = Arc::clone(&game);
        let udp_client_clone = Arc::clone(&udp_client);

        let receive_update_handle = tokio::spawn(async move {
            while receive_updates_clone.load(Ordering::Relaxed) {
                match udp_client_clone.recv_updated_game() {
                    Ok(updated_game) => {
                        match game_clone.lock() {
                            Ok(mut current_game) => {
                                *current_game = updated_game;
                            }
                            Err(_) => {
                                // TODO: Most likely ignore this error?
                            }
                        }
                    }
                    Err(_) => {
                        // TODO: Most likely ignore this error?
                    }
                }
            }
        });

        Self {
            game,
            our_player_id,
            receive_update_handle,
            receive_updates,
            udp_client,
        }
    }

    async fn send_player_move(&self, direction: Direction) -> Result<(), std::io::Error> {
        if let Ok(game) = self.game.lock() {
            let client_input = ClientInput::new(
                game.id.to_string(),
                self.our_player_id.to_string(),
                ClientInputType::MovePaddle(direction),
            );
            self.udp_client
                .send_client_input(client_input)
                .expect("Failed to send input"); // TODO: Handle this error better
        }
        Ok(())
    }
}

impl State for GameBoard {}

#[async_trait::async_trait]
impl Update for GameBoard {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Up | KeyCode::Char('w') => {
                    self.send_player_move(Direction::Positive).await?;
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    self.send_player_move(Direction::Negative).await?;
                }
                KeyCode::Left | KeyCode::Char('a') => {
                    self.send_player_move(Direction::Negative).await?;
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    self.send_player_move(Direction::Positive).await?;
                }
                KeyCode::Esc => {
                    // TODO: just a placeholder for now
                    return Ok(Some(Box::new(Menu::new(0))));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for GameBoard {
    fn render(&self, frame: &mut Frame) {
        if let Ok(game) = self.game.lock() {
            // Calculate the game area and scaling factors once
            let (game_area, scale_x, scale_y) = calculate_game_area(frame);

            let block = Block::bordered();

            frame.render_widget(block, game_area);

            // Render players
            let players: Vec<&PlayerDto> = game.players.values().collect();
            render_players(
                &players,
                self.our_player_id,
                frame,
                &game_area,
                scale_x,
                scale_y,
            );

            // Render the ball
            if let Some(ball) = &game.ball {
                render_ball(ball, frame, &game_area, scale_x, scale_y);
            }
        }
    }
}

impl Drop for GameBoard {
    fn drop(&mut self) {
        self.receive_updates.store(false, Ordering::Relaxed);
        self.receive_update_handle.abort();
    }
}
