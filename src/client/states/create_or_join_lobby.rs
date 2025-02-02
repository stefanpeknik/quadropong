use crate::client::config;
use crate::client::net::tcp::TcpClient;

use super::lobby::Lobby;
use super::menu::Menu;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::input::Input;
use super::utils::render::{into_title, render_inner_rectangle, render_outer_rectangle};
use super::utils::widget::WidgetTrait;

use crossterm::event::KeyCode;
use log::{error, info};
use ratatui::layout::{Constraint, Flex, Layout, Position};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

#[derive(PartialEq)]
pub enum Options {
    Create,
    Join,
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Create => write!(f, " {} ", into_title("create lobby")),
            Options::Join => write!(f, " {} ", into_title("join lobby")),
        }
    }
}

pub struct CreateOrJoinLobby {
    options: Vec<Options>,
    selected: usize,
    join_lobby_input: Input,
    error_message: Option<String>,
    tcp_client: TcpClient,
    config: config::Config,
}

impl CreateOrJoinLobby {
    pub fn new(config: config::Config) -> Self {
        Self {
            options: vec![Options::Create, Options::Join],
            selected: 0,
            join_lobby_input: Input::new(),
            error_message: None,
            tcp_client: TcpClient::new(&config.api_url),
            config,
        }
    }

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

impl State for CreateOrJoinLobby {}

impl HasConfig for CreateOrJoinLobby {
    fn config(&self) -> config::Config {
        self.config.clone()
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
                    info!("Moving from CreateOrJoinLobby to Menu");
                    return Ok(Some(Box::new(Menu::new(0, self.config.clone()))));
                }

                _ => {}
            };
            // match keys for the selected option
            match self.options[self.selected] {
                Options::Create => match key_code {
                    KeyCode::Enter => match self.tcp_client.create_game().await {
                        // Game is created, but we need to join it to get our player id
                        Ok(game) => match self
                            .tcp_client
                            .join_game(game.id, Some(self.config.player_name.clone()))
                            .await
                        {
                            // We successfully joined the game
                            Ok(our_player) => {
                                info!("Moving from CreateOrJoinLobby to Lobby via create, game id: {:?}, our player id: {:?}", game.id, our_player.id);
                                return Ok(Some(Box::new(Lobby::new(
                                    game,
                                    our_player.id,
                                    self.config.clone(),
                                ))));
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
                    KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Char(_)
                    | KeyCode::Backspace
                    | KeyCode::Tab => self.join_lobby_input.handle_key_event(key_code),
                    KeyCode::Enter => {
                        match uuid::Uuid::parse_str(&self.join_lobby_input.input) {
                            Ok(inputted_game_id) => {
                                match self.tcp_client.get_game(inputted_game_id).await {
                                    Ok(game) => match self
                                        .tcp_client
                                        .join_game(game.id, Some(self.config.player_name.clone()))
                                        .await
                                    {
                                        Ok(our_player) => {
                                            info!("Moving from CreateOrJoinLobby to Lobby via join, game id: {:?}, our player id: {:?}", game.id, our_player.id);
                                            return Ok(Some(Box::new(Lobby::new(
                                                game,
                                                our_player.id,
                                                self.config.clone(),
                                            ))));
                                        }
                                        Err(e) => {
                                            info!("Error joining game: {}", e);
                                            self.error_message = Some(e.to_string());
                                        }
                                    },
                                    Err(e) => {
                                        error!("Error getting game: {}", e);
                                        self.error_message = Some(e.to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Invalid UUID when joining game: {}", e);
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
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong ",
            vec![
                " Back".into(),
                " <Esc> ".light_blue().bold(),
                "| Up".into(),
                " <\u{2191}> ".light_blue().into(),
                "| Down".into(),
                " <\u{2193}> ".light_blue().into(),
            ],
        );

        let inner_rect = render_inner_rectangle(frame, outer_rect);

        let [create_area, join_area] =
            Layout::vertical(vec![Constraint::Length(1), Constraint::Length(3)])
                .flex(Flex::SpaceAround)
                .areas(inner_rect);
        let [_, error_area, _] = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .areas(inner_rect);

        // render create lobby area
        let create_area_text = if self.options[self.selected] == Options::Create {
            Line::from(format!(">{}<", Options::Create)).bold()
        } else {
            Line::from(Options::Create.to_string())
        };
        frame.render_widget(Paragraph::new(create_area_text).centered(), create_area);

        let block_width_layout = Layout::horizontal(vec![
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ]);

        // render join lobby area
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
                inner_join_input_area.x + self.join_lobby_input.char_index.try_into().unwrap_or(0),
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
