use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::game_models::client_input::{ClientInput, ClientInputType};
use crate::game_models::game::Game;
use crate::net::udp::UdpClient;

use super::game_board::GameBoard;
use super::menu::Menu;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::render::{render_inner_rectangle, render_list, render_outer_rectangle};

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;
use uuid::Uuid;

pub enum Options {
    TODO,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::TODO]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::TODO => write!(f, "TODO"),
        }
    }
}

pub struct Lobby {
    options: Vec<Options>,
    selected: usize,
    game: Arc<Mutex<Game>>,
    our_player_id: Uuid,
    receive_updates: Arc<AtomicBool>,
    receive_update_handle: tokio::task::JoinHandle<()>,
    udp_client: Arc<UdpClient>,
}

impl Lobby {
    pub fn new(game: Game, our_player_id: Uuid) -> Self {
        let udp_client = Arc::new(UdpClient::new().expect("Failed to create UDP client")); // TODO: Handle this error

        // send introduction message
        let client_input = ClientInput::new(
            game.id.to_string(),
            our_player_id.to_string(),
            ClientInputType::JoinGame,
        );
        udp_client
            .send_client_input(client_input)
            .expect("Failed to send introduction message"); // TODO: Handle this error

        let receive_updates = Arc::new(AtomicBool::new(true));
        let game = Arc::new(Mutex::new(game));

        // Start a task to receive updates
        let udp_client_clone = Arc::clone(&udp_client);
        let receive_updates_clone = Arc::clone(&receive_updates);
        let game_clone = Arc::clone(&game);
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
            options: Options::list(),
            selected: 0,
            game,
            our_player_id,
            udp_client,
            receive_updates,
            receive_update_handle,
        }
    }
}

impl HasOptions for Lobby {
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

impl State for Lobby {}

#[async_trait::async_trait]
impl Update for Lobby {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Enter => match self.options[self.selected] {
                    // TODO: Implement this
                    _ => {
                        // TODO: for now send into game
                        return Ok(Some(Box::new(GameBoard::new(
                            self.game
                                .lock()
                                .expect("Failed to lock game") // TODO: Handle this error
                                .to_owned(),
                            self.our_player_id,
                            Arc::clone(&self.udp_client),
                        ))));
                    }
                },
                KeyCode::Esc => {
                    return Ok(Some(Box::new(Menu::new(0))));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for Lobby {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong ",
            vec![" Back ".into(), " <Esc> ".blue().bold()],
        );

        let inner_rect = render_inner_rectangle(frame, outer_rect);

        if let Ok(game) = self.game.lock() {
            let mut list = vec![game.id.to_string()];
            let mut players: Vec<_> = game
                .players
                .iter()
                .map(|(p_id, p)| format!("{}: {}", p_id, p.name))
                .collect();
            players.sort();
            list.extend(players);
            list.push(format!("You: {}", self.our_player_id));

            render_list(frame, &list, self.selected, inner_rect);
        }
    }
}

impl Drop for Lobby {
    fn drop(&mut self) {
        // Signal the task to stop
        self.receive_updates.store(false, Ordering::Relaxed);

        self.receive_update_handle.abort();
    }
}
