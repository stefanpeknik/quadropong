use std::sync::{Arc, Mutex};

use crate::client::config;
use crate::client::error::ClientError;
use crate::client::states::game_end::GameEnd;
use crate::common::models::{Direction, GameDto, GameState};
use crate::common::{Game, Player, PlayerPosition};

use super::menu::Menu;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::render::render_game;

use axum::async_trait;
use crossterm::event::KeyCode;
use log::{error, info};
use rand::seq::SliceRandom;
use ratatui::Frame;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct Training {
    config: config::Config,
    game: Arc<Mutex<Game>>,
    our_player_id: Uuid,
    cancellation_token: CancellationToken,
    _game_tick_handle: tokio::task::JoinHandle<()>,
}

impl Training {
    pub fn new(config: config::Config) -> Result<Self, ClientError> {
        let mut game = Game::new();
        let mut our_player = Player::new(config.player_name.clone(), false);
        our_player.is_ready = true;
        let our_player_id = our_player.id;
        let mut players = [
            our_player,
            Player::new("bot".to_string(), true),
            Player::new("bot".to_string(), true),
            Player::new("bot".to_string(), true),
        ];
        let mut rng = rand::rng();
        players.shuffle(&mut rng);

        for mut player in players {
            if let Some(position) = game.assign_position() {
                player.position = Some(position);
            }
            let _ = game.add_player(player);
        }

        let game = Arc::new(Mutex::new(game));
        let cancellation_token = CancellationToken::new();

        let game_clone = game.clone();
        let cancellation_token_clone = cancellation_token.clone();
        let game_tick_handle = tokio::spawn(async move {
            let _ = game_clone.lock().expect("Failed to lock game").start_game();
            loop {
                tokio::select! {
                     _ = cancellation_token_clone.cancelled() => break,
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000 / 60)) => {
                        if let Ok(mut g) = game_clone.lock() {
                            g.game_tick();
                            let ball = g.ball.clone();
                            for player in g.players.values_mut() {
                                if player.is_ai {
                                    if let Some(ref ball) = ball {
                                        player.ai(ball.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            config,
            game,
            our_player_id,
            cancellation_token,
            _game_tick_handle: game_tick_handle,
        })
    }
}

impl State for Training {}

impl HasConfig for Training {
    fn config(&self) -> config::Config {
        self.config.clone()
    }
}

#[async_trait]
impl Update for Training {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, ClientError> {
        if let Ok(game) = self.game.lock() {
            if game.state == GameState::Finished {
                info!("Game finished");
                info!("Moving from Training to GameEnd");
                return Ok(Some(Box::new(GameEnd::new(
                    GameDto::from(game.clone()),
                    self.our_player_id,
                    self.config.clone(),
                )?)));
            }
        } else {
            error!("Failed to lock game");
        }

        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Esc => {
                    log::info!("Moving from Training to Menu");
                    return Ok(Some(Box::new(Menu::new(1, self.config.clone())?)));
                }
                _ => {
                    if let Ok(mut game) = self.game.lock() {
                        match game
                            .players
                            .values_mut()
                            .find(|p| p.id == self.our_player_id)
                        {
                            Some(player) => {
                                if let Some(position) = player.position {
                                    match position {
                                        PlayerPosition::Top | PlayerPosition::Bottom => {
                                            match key_code {
                                                KeyCode::Left => {
                                                    player.move_paddle(Direction::Negative);
                                                }
                                                KeyCode::Right => {
                                                    player.move_paddle(Direction::Positive);
                                                }
                                                _ => {}
                                            }
                                        }
                                        PlayerPosition::Left | PlayerPosition::Right => {
                                            match key_code {
                                                KeyCode::Up => {
                                                    player.move_paddle(Direction::Negative);
                                                }
                                                KeyCode::Down => {
                                                    player.move_paddle(Direction::Positive);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }
            };
        }
        Ok(None)
    }
}

impl Render for Training {
    fn render(&self, frame: &mut Frame) {
        if let Ok(game) = self.game.lock() {
            render_game(
                &GameDto::from(game.clone()),
                self.our_player_id,
                self.config.player_color,
                self.config.other_players_color,
                frame,
            );
        } else {
            error!("Failed to lock game");
        }
    }
}

impl Drop for Training {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
