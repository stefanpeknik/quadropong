use crate::client::net::udp::UdpClient;
use crate::client::settings;
use crate::common::models::{
    ClientInput, ClientInputType, Direction, GameDto, GameState, PlayerDto,
};
use crate::common::PlayerPosition;

use super::game_end::GameEnd;
use super::traits::{HasSettings, Render, State, Update};
use super::utils::render::{calculate_game_area, render_ball, render_player};

use crossterm::event::KeyCode;
use ratatui::widgets::Block;
use ratatui::Frame;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

pub struct GameBoard {
    game: Arc<Mutex<GameDto>>,
    our_player_id: Uuid,
    our_player_position: PlayerPosition,
    receive_updates: Arc<AtomicBool>,
    receive_update_handle: tokio::task::JoinHandle<()>,
    udp_client: Arc<UdpClient>,
    settings: settings::Settings,
}

impl GameBoard {
    pub fn new(
        game: GameDto,
        our_player_id: Uuid,
        udp_client: Arc<UdpClient>,
        settings: settings::Settings,
    ) -> Self {
        let our_player_position = game
            .players
            .get(&our_player_id)
            .map(|player| player.position.unwrap_or(PlayerPosition::Left)) // TODO: Handle this error better
            .unwrap_or(PlayerPosition::Left); // TODO: Handle this error better
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
            our_player_position,
            receive_update_handle,
            receive_updates,
            udp_client,
            settings,
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

impl HasSettings for GameBoard {
    fn settings(&self) -> settings::Settings {
        self.settings.clone()
    }
}

#[async_trait::async_trait]
impl Update for GameBoard {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        match self.game.lock() {
            Ok(game) => {
                if game.state == GameState::Finished {
                    return Ok(Some(Box::new(GameEnd::new(
                        game.clone(),
                        self.our_player_id,
                        self.settings.clone(),
                    ))));
                }
            }
            Err(_) => {}
        }
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Esc => {
                    // TODO: just a placeholder for now
                    match self.game.lock() {
                        Ok(game) => {
                            return Ok(Some(Box::new(GameEnd::new(
                                game.clone(),
                                self.our_player_id,
                                self.settings.clone(),
                            ))));
                        }
                        Err(_) => {}
                    }
                }
                _ => match self.our_player_position {
                    PlayerPosition::Left | PlayerPosition::Right => match key_code {
                        KeyCode::Up | KeyCode::Char('w') => {
                            self.send_player_move(Direction::Negative).await?;
                        }
                        KeyCode::Down | KeyCode::Char('s') => {
                            self.send_player_move(Direction::Positive).await?;
                        }
                        _ => {}
                    },
                    PlayerPosition::Top | PlayerPosition::Bottom => match key_code {
                        KeyCode::Right | KeyCode::Char('d') => {
                            self.send_player_move(Direction::Positive).await?;
                        }
                        KeyCode::Left | KeyCode::Char('a') => {
                            self.send_player_move(Direction::Negative).await?;
                        }
                        _ => {}
                    },
                },
            };
        }
        Ok(None)
    }
}

impl Render for GameBoard {
    fn render(&self, frame: &mut Frame) {
        if let Ok(game) = self.game.lock() {
            // Calculate the game area and scaling factors once
            let (game_area_bounding_box, game_area, scale_x, scale_y) = calculate_game_area(&frame);

            // Render the game area border
            frame.render_widget(
                Block::bordered().title_bottom("press Enter to \"finish\" the game"), // TODO
                game_area_bounding_box,
            );

            // Render players
            for player in game.players.values() {
                let player_color = if player.id == self.our_player_id {
                    self.settings.player_color
                } else {
                    self.settings.other_players_color
                };
                render_player(player, player_color, frame, &game_area, scale_x, scale_y);
            }

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
