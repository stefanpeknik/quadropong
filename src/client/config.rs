use chrono::Utc;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};

use super::states::{
    settings::Options,
    utils::widget::{get_widget_text, Widget},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_api_addr")]
    pub api_url: String,
    #[serde(default = "default_socket_addr")]
    pub socket_addr: String,
    pub player_name: String,
    pub player_color: Color,
    pub other_players_color: Color,
    pub fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            player_name: "player".to_string(),
            api_url: default_api_addr(),
            socket_addr: default_socket_addr(),
            player_color: Color::Green,
            other_players_color: Color::White,
            fps: 60,
        }
    }
}

pub fn default_api_addr() -> String {
    "http://127.0.0.1:3000".to_string()
}

pub fn default_socket_addr() -> String {
    "127.0.0.1:34254".to_string()
}

impl Config {
    pub fn to_vec(&self) -> Vec<String> {
        vec![
            self.player_name.clone(),
            self.api_url.clone(),
            self.socket_addr.clone(),
            self.player_color.to_string(),
            self.other_players_color.to_string(),
            self.fps.to_string(),
        ]
    }

    pub fn get_config_path() -> Option<path::PathBuf> {
        if let Some(mut config_dir) = dirs::config_local_dir() {
            config_dir.push("quadropong");
            fs::create_dir_all(&config_dir).ok()?;
            config_dir.push("settings.conf");
            Some(config_dir)
        } else {
            None
        }
    }

    pub fn save_config(&mut self) -> std::io::Result<()> {
        if let Some(config_path) = Self::get_config_path() {
            let mut file = fs::File::create(config_path)?;
            let config_data = serde_json::to_string_pretty(self)?;
            io::Write::write_all(&mut file, config_data.as_bytes())?;
        }

        Ok(())
    }

    pub fn load_config() -> Result<Config, io::Error> {
        if let Some(config_path) = Self::get_config_path() {
            let config_data = if config_path.exists() {
                let data = std::fs::read_to_string(&config_path)?;
                if data.is_empty() {
                    None
                } else {
                    Some(data)
                }
            } else {
                None
            };

            match config_data {
                Some(data) => match serde_json::from_str::<Config>(&data) {
                    Ok(settings) => Ok(settings),
                    Err(_e) => {
                        // When serde fails load default and save old settings to recoverable file
                        Self::save_failed_config(&config_path);
                        let mut default_settings = Self::default();
                        default_settings.save_config()?;
                        Ok(default_settings)
                    }
                },
                None => {
                    // Empty or nonexistent settings.conf
                    let mut default_settings = Self::default();
                    default_settings.save_config()?;
                    Ok(default_settings)
                }
            }
        } else {
            // No .config file PATH
            Ok(Self::default())
        }
    }

    fn save_failed_config(config_path: &path::PathBuf) {
        let timestamp = Utc::now().format("%m%d_%H%M").to_string();

        let mut failed_config_path = config_path.clone();
        failed_config_path.set_file_name(format!("failed_settings_{}.conf", timestamp));

        // dont really care if it fails
        let _ = fs::copy(config_path, failed_config_path);
    }

    pub fn save_option(&mut self, selected_option: &Options) {
        match selected_option {
            Options::PlayerName(widget) => {
                self.player_name = get_widget_text(widget);
            }
            Options::PlayerColor(widget) => {
                if let Widget::Slider(slider) = widget {
                    self.player_color = slider.clone().get_color();
                }
            }
            Options::OtherPlayersColor(widget) => {
                if let Widget::Slider(slider) = widget {
                    self.other_players_color = slider.clone().get_color();
                }
            }
            Options::FPS(widget) => {
                if let Ok(number) = get_widget_text(widget).parse() {
                    self.fps = number;
                }
            }
        }
    }
}
