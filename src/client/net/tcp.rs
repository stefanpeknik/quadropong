use reqwest::Client;
use serde_json;
use uuid::Uuid;

use crate::common::{Game, JoinGameRequest, Player};

use super::error::TcpError;

pub struct TcpClient {
    server_addr: String,
    client: Client,
}

impl TcpClient {
    pub fn new(server_addr: &str) -> Self {
        TcpClient {
            server_addr: server_addr.to_string(),
            client: Client::new(),
        }
    }

    pub async fn create_game(&self) -> Result<Game, TcpError> {
        let url = format!("{}/game", self.server_addr);

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
        let url = format!("{}/game/{}", self.server_addr, game_id);

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

    pub async fn join_game(
        &self,
        game_id: Uuid,
        username: Option<String>,
    ) -> Result<Player, TcpError> {
        let url = format!("{}/game/{}/join", self.server_addr, game_id);
        let payload_json = serde_json::to_string(&JoinGameRequest { username })?;

        // Send the request and handle potential errors
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(payload_json)
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
