use crate::game_models::game::Game;
use crate::game_models::player::{Player, PlayerPosition};
use crate::net::tcp::get_game;

use super::menu::Menu;
use super::traits::{Render, State, Update};
use super::utils::render::{draw_inner_rectangle, draw_outer_rectangle, render_list};

use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use uuid::Uuid;

#[derive(Clone)]
pub struct GameBoard {
    game: Game,
    our_player_id: Uuid,
}

impl GameBoard {
    pub fn new(game: Game, our_player_id: Uuid) -> Self {
        Self {
            game,
            our_player_id,
        }
    }
}

impl State for GameBoard {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

#[async_trait::async_trait]
impl Update for GameBoard {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        match get_game(self.game.id).await {
            Ok(game) => {
                self.game = game;
            }
            Err(e) => {
                // TODO: Handle this error
            }
        }
        if let Some(key_code) = key_code {
            match key_code {
                // TODO: Handle key presses
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for GameBoard {
    fn render(&self, frame: &mut Frame) {}
}

fn render_player(player: &Player, is_our_player: bool, frame: &mut Frame) {
    // Get the terminal size and calculate the game area
    let terminal_size = frame.area();
    let game_area_width = terminal_size.width.min(terminal_size.height);
    let game_area = Rect {
        x: (terminal_size.width - game_area_width) / 2,
        y: (terminal_size.height - game_area_width) / 2,
        width: game_area_width,
        height: game_area_width,
    };

    // Map the 10x10 game space to the terminal's game area
    let scale_x = game_area.width as f32 / 10.0;
    let scale_y = game_area.height as f32 / 10.0;

    // Calculate paddle dimensions and position
    let paddle_length = (player.paddle_width * 2.0 * scale_x) as u16;
    let paddle_thickness = 1; // Paddle depth is 1 character
    let paddle_center = (player.paddle_position * scale_x) as u16;

    // Determine paddle position based on player side
    match player.position {
        Some(PlayerPosition::Top) => {
            let paddle_x = game_area.x + paddle_center - paddle_length / 2;
            let paddle_y = game_area.y;
            frame.render_widget(
                Paragraph::new("─".repeat(paddle_length as usize))
                    .style(Style::default().fg(Color::White)),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_length,
                    height: paddle_thickness,
                },
            );
        }
        Some(PlayerPosition::Bottom) => {
            let paddle_x = game_area.x + paddle_center - paddle_length / 2;
            let paddle_y = game_area.y + game_area.height - paddle_thickness;
            frame.render_widget(
                Paragraph::new("─".repeat(paddle_length as usize))
                    .style(Style::default().fg(Color::White)),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_length,
                    height: paddle_thickness,
                },
            );
        }
        Some(PlayerPosition::Left) => {
            let paddle_x = game_area.x;
            let paddle_y = game_area.y + paddle_center - paddle_length / 2;
            frame.render_widget(
                Paragraph::new("│".repeat(paddle_length as usize))
                    .style(Style::default().fg(Color::White)),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_thickness,
                    height: paddle_length,
                },
            );
        }
        Some(PlayerPosition::Right) => {
            let paddle_x = game_area.x + game_area.width - paddle_thickness;
            let paddle_y = game_area.y + paddle_center - paddle_length / 2;
            frame.render_widget(
                Paragraph::new("│".repeat(paddle_length as usize))
                    .style(Style::default().fg(Color::White)),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_thickness,
                    height: paddle_length,
                },
            );
        }
        None => {}
    }
}
