use reqwest::Client;
use uuid::Uuid;

use crate::common::{Game, Player};

use super::error::TcpError;

const SERVER_ADDR: &str = "http://127.0.0.1:3000";

pub struct TcpClient {
    client: Client,
}

impl TcpClient {
    pub fn new() -> Self {
        TcpClient {
            client: Client::new(),
        }
    }

    pub async fn create_game(&self) -> Result<Game, TcpError> {
        let url = format!("{}/game", SERVER_ADDR);

        // Send the request and handle potential errors
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(TcpError::FailedToSendRequest)?;

        // Check if the response status is successful
        if !response.status().is_success() {
            return Err(TcpError::ServerError(format!(
                "Server returned status code: {}",
                response.status()
            )));
        }

        // Read the response body and handle potential errors
        let response_text = response
            .text()
            .await
            .map_err(TcpError::FailedToReadResponse)?;

        // Deserialize the response and handle potential errors
        let game: Game = serde_json::from_str(&response_text)?;

        Ok(game)
    }

    pub async fn get_game(&self, game_id: Uuid) -> Result<Game, TcpError> {
        let url = format!("{}/game/{}", SERVER_ADDR, game_id);

        // Send the request and handle potential errors
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TcpError::FailedToSendRequest)?;

        // Check if the response status is successful
        if !response.status().is_success() {
            return Err(TcpError::ServerError(format!(
                "Server returned status code: {}",
                response.status()
            )));
        }

        // Read the response body and handle potential errors
        let response_text = response
            .text()
            .await
            .map_err(TcpError::FailedToReadResponse)?;

        // Deserialize the response and handle potential errors
        let game: Game = serde_json::from_str(&response_text)?;

        Ok(game)
    }

    pub async fn join_game(&self, game_id: Uuid) -> Result<Player, TcpError> {
        let url = format!("{}/game/{}/join", SERVER_ADDR, game_id);

        // Send the request and handle potential errors
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await
            .map_err(TcpError::FailedToSendRequest)?;

        // Check if the response status is successful
        if !response.status().is_success() {
            return Err(TcpError::ServerError(format!(
                "Server returned status code: {}",
                response.status()
            )));
        }

        // Read the response body and handle potential errors
        let response_text = response
            .text()
            .await
            .map_err(TcpError::FailedToReadResponse)?;

        // Deserialize the response and handle potential errors
        let player: Player = serde_json::from_str(&response_text)?;

        Ok(player)
    }
}
