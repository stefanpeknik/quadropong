use std::sync::Mutex;

use crate::client::config;

use super::menu::Menu;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::input::Input;
use super::utils::render::{into_title, render_outer_rectangle, render_settings};
use super::utils::slider::Slider;
use super::utils::widget::{Widget, WidgetTrait};

use axum::async_trait;
use crossterm::event::KeyCode;
use log::{error, info};
use ratatui::layout::Margin;
use ratatui::style::Stylize;
use ratatui::Frame;

pub enum Options {
    PlayerName(Widget),
    PlayerColor(Widget),
    OtherPlayersColor(Widget),
    FPS(Widget),
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::PlayerName(_) => write!(f, " {} ", into_title("player name")),
            Options::PlayerColor(_) => write!(f, " {} ", into_title("plyer color")),
            Options::OtherPlayersColor(_) => write!(f, " {} ", into_title("other player color")),
            Options::FPS(_) => write!(f, " {} ", into_title("fps")),
        }
    }
}

pub struct Settings {
    options: Vec<Options>,
    selected: usize,
    config: Mutex<config::Config>,
}

impl Settings {
    pub fn new(config: config::Config) -> Self {
        let options = Self::fill_settings(config.clone());
        Self {
            options,
            selected: 0,
            config: Mutex::new(config),
        }
    }

    fn fill_settings(settings: config::Config) -> Vec<Options> {
        vec![
            Options::PlayerName(Widget::Input(Input::from(settings.player_name.to_string()))),
            Options::PlayerColor(Widget::Slider(Slider::from(
                settings.player_color.to_string(),
            ))),
            Options::OtherPlayersColor(Widget::Slider(Slider::from(
                settings.other_players_color.to_string(),
            ))),
            Options::FPS(Widget::Input(Input::from(settings.fps.to_string()))),
        ]
    }

    pub fn get_widget_active(&self) -> &Widget {
        match &self.options[self.selected] {
            Options::PlayerName(widget) => widget,
            Options::PlayerColor(widget) => widget,
            Options::OtherPlayersColor(widget) => widget,
            Options::FPS(widget) => widget,
        }
    }

    pub fn get_widget_all(&self) -> Vec<&Widget> {
        self.options
            .iter()
            .map(|option| match option {
                Options::PlayerName(widget) => widget,
                Options::PlayerColor(widget) => widget,
                Options::OtherPlayersColor(widget) => widget,
                Options::FPS(widget) => widget,
            })
            .collect()
    }

    pub fn get_widget_active_as_mut(&mut self) -> &mut Widget {
        match &mut self.options[self.selected] {
            Options::PlayerName(widget) => widget,
            Options::PlayerColor(widget) => widget,
            Options::OtherPlayersColor(widget) => widget,
            Options::FPS(widget) => widget,
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

impl State for Settings {}

impl HasConfig for Settings {
    fn config(&self) -> config::Config {
        self.config.lock().unwrap().clone() // TODO: Check if this is correct
    }
}

#[async_trait]
impl Update for Settings {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        let active_widget = self.get_widget_active_as_mut();

        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Left
                | KeyCode::Right
                | KeyCode::Char(_)
                | KeyCode::Backspace
                | KeyCode::Tab => {
                    match active_widget {
                        Widget::Slider(ref mut slider) => {
                            slider.handle_key_event(key_code);
                        }
                        Widget::Input(ref mut input) => {
                            input.handle_key_event(key_code);
                        }
                    }
                    if let Ok(mut settings) = self.config.lock() {
                        // save selected option to settings
                        settings.save_option(&self.options[self.selected]);
                    } else {
                        error!("Failed to lock settings");
                    }
                }
                KeyCode::Esc => {
                    if let Ok(mut settings) = self.config.lock() {
                        // save to config file before exiting screen
                        let _ = settings.save_config();
                        info!("Config saved");
                        info!("Moving from Settings to Menu");
                        return Ok(Some(Box::new(Menu::new(2, settings.clone()))));
                    }
                }
                KeyCode::End => {
                    if let Ok(mut settings) = self.config.lock() {
                        // load default settings
                        *settings = config::Config::default();
                        self.options = Self::fill_settings(config::Config::default());
                    }
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for Settings {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong - Settings ",
            vec![
                " Back".into(),
                " <Esc> ".light_blue().bold(),
                "| Up".into(),
                " <\u{2191}> ".light_blue().bold(),
                "| Down".into(),
                " <\u{2193}> ".light_blue().bold(),
                "| Reset default".into(),
                " <End> ".light_blue().bold(),
            ],
        );

        let inner_rect = outer_rect.inner(Margin {
            horizontal: 5,
            vertical: 5,
        });

        if let Ok(_settings) = self.config.lock() {
            let selected_index = self.selected;
            render_settings(
                frame,
                &self
                    .options
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
                &self.get_widget_all(),
                &self.get_widget_active(),
                selected_index,
                inner_rect,
            );
        }
    }
}
