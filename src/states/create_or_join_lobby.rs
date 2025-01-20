use crate::net::tcp::{create_game, get_game, join_game};

use super::lobby::Lobby;
use super::menu::Menu;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::input::Input;
use super::utils::render::{
    draw_inner_rectangle, draw_outer_rectangle, evenly_distanced_rects,
    render_text_in_center_of_rect,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

#[derive(Clone, PartialEq)]
pub enum Options {
    Create,
    Join,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::Create, Options::Join]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Create => write!(f, "C R E A T E  L O B B Y"),
            Options::Join => write!(f, " J O I N  L O B B Y "),
        }
    }
}

#[derive(Clone)]
pub struct CreateOrJoinLobby {
    options: Vec<Options>,
    selected: usize,
    join_lobby_input: Input,
    error_message: Option<String>,
}

impl CreateOrJoinLobby {
    pub fn new() -> Self {
        Self {
            options: Options::list(),
            selected: 0,
            join_lobby_input: Input::new(),
            error_message: None,
        }
    }
}

impl HasOptions for CreateOrJoinLobby {
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

impl State for CreateOrJoinLobby {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

#[async_trait::async_trait]
impl Update for CreateOrJoinLobby {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            // match navigation keys between options/states
            match key_code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Esc => {
                    return Ok(Some(Box::new(Menu::new(0))));
                }

                _ => {}
            };
            // match keys for the selected option
            match self.options[self.selected] {
                Options::Create => match key_code {
                    KeyCode::Enter => match create_game().await {
                        // Game is created, but we need to join it to get our player id
                        Ok(game) => match join_game(game.id).await {
                            // We successfully joined the game
                            Ok(our_player) => {
                                return Ok(Some(Box::new(Lobby::new(game, our_player.id))));
                            }
                            Err(e) => {
                                self.error_message = Some(e.to_string());
                            }
                        },
                        Err(e) => {
                            self.error_message = Some(e.to_string());
                        }
                    },
                    _ => {}
                },
                Options::Join => match key_code {
                    KeyCode::Left => {
                        self.join_lobby_input.move_left();
                    }
                    KeyCode::Right => {
                        self.join_lobby_input.move_right();
                    }
                    KeyCode::Char(c) => {
                        self.join_lobby_input.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        self.join_lobby_input.delete_char();
                    }
                    KeyCode::Tab => {
                        if let Ok(mut ctx) = ClipboardContext::new() {
                            if let Ok(clipboard_content) = ctx.get_contents() {
                                self.join_lobby_input.insert_clipboard(clipboard_content);
                            }
                        }
                    }
                    KeyCode::Enter => {
                        match uuid::Uuid::parse_str(&self.join_lobby_input.input) {
                            Ok(inputted_game_id) => match get_game(inputted_game_id).await {
                                Ok(game) => match join_game(game.id).await {
                                    Ok(our_player) => {
                                        return Ok(Some(Box::new(Lobby::new(game, our_player.id))));
                                    }
                                    Err(e) => {
                                        self.error_message = Some(e.to_string());
                                    }
                                },
                                Err(e) => {
                                    self.error_message = Some(e.to_string());
                                }
                            },
                            Err(e) => {
                                self.error_message = Some(format!("Invalid UUID: {}", e));
                            }
                        };
                    }
                    _ => {}
                },
            }
        }
        Ok(None)
    }
}

impl Render for CreateOrJoinLobby {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = draw_outer_rectangle(
            frame,
            " quadropong ",
            vec![" Back ".into(), "<Esc> ".light_blue().bold()],
        );

        let inner_rect = draw_inner_rectangle(frame, outer_rect);

        let layout = Layout::vertical(vec![
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(5),
            Constraint::Percentage(20),
            Constraint::Percentage(5),
        ]);

        let [create_area, _, join_area, _, error_area, _] = layout.areas(inner_rect);

        // render create lobby area
        let create_area_text = if self.options[self.selected] == Options::Create {
            Line::from(format!("> {} <", Options::Create)).bold()
        } else {
            Line::from(Options::Create.to_string())
        };
        frame.render_widget(
            Paragraph::new(create_area_text).centered(),
            evenly_distanced_rects(create_area, 2)[1],
        );

        // render join lobby area
        let block_width_layout = Layout::horizontal(vec![
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ]);
        let [_, join_input_area, _] = block_width_layout.areas(join_area);
        let join_area_text = if self.options[self.selected] == Options::Join {
            Line::from(format!(" >{}< ", Options::Join))
                .bold()
                .centered()
        } else {
            Line::from(Options::Join.to_string()).centered()
        };
        let join_input_block = Block::bordered().title(join_area_text).title_bottom(
            Line::from(vec![
                " Join ".into(),
                "<Enter>".green().bold(),
                " | Paste ".into(),
                "<TAB> ".green().bold(),
            ])
            .centered(),
        );
        let inner_join_input_area = join_input_block.inner(join_input_area);
        let mut style = Style::default();
        if self.options[self.selected] == Options::Join {
            frame.set_cursor_position(Position::new(
                inner_join_input_area.x + self.join_lobby_input.char_index as u16, // TODO the `as` seems sus here
                inner_join_input_area.y,
            ));
            style = Style::default().bold();
        }
        frame.render_widget(join_input_block.style(style), join_input_area);
        frame.render_widget(
            Paragraph::new(self.join_lobby_input.input.clone()),
            inner_join_input_area,
        );

        // render error message area
        if let Some(error_message) = &self.error_message {
            let error_layout =
                Layout::vertical(vec![Constraint::Percentage(80), Constraint::Percentage(20)]);
            let [error_message_area, _] = error_layout.areas(error_area);
            let [_, error_message_area, _] = block_width_layout.areas(error_message_area);
            frame.render_widget(
                Paragraph::new(error_message.clone())
                    .red()
                    .centered()
                    .wrap(Wrap { trim: true }),
                error_message_area,
            );
        }
    }
}
