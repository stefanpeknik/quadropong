use reqwest::Client;
use thiserror::Error;
use uuid::Uuid;

use crate::game_models::{game::Game, player::Player};

const SERVER_ADDR: &str = "http://127.0.0.1:3000";

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to send request: {0}")]
    FailedToSendRequest(#[from] reqwest::Error),
    #[error("Failed to read response: {0}")]
    FailedToReadResponse(reqwest::Error),
    #[error("Failed to deserialize response: {0}")]
    FailedToDeserializeResponse(#[from] serde_json::Error),
    #[error("Server returned an error: {0}")]
    ServerError(String),
}

pub async fn create_game() -> Result<Game, NetError> {
    let url = format!("{}/game", SERVER_ADDR);
    let client = Client::new();

    // Send the request and handle potential errors
    let response = client
        .post(&url)
        .send()
        .await
        .map_err(NetError::FailedToSendRequest)?;

    // Check if the response status is successful
    if !response.status().is_success() {
        return Err(NetError::ServerError(format!(
            "Server returned status code: {}",
            response.status()
        )));
    }

    // Read the response body and handle potential errors
    let response_text = response
        .text()
        .await
        .map_err(NetError::FailedToReadResponse)?;

    // Deserialize the response and handle potential errors
    let game: Game = serde_json::from_str(&response_text)?;

    Ok(game)
}

pub async fn get_game(game_id: Uuid) -> Result<Game, NetError> {
    let url = format!("{}/game/{}", SERVER_ADDR, game_id);
    let client = Client::new();

    // Send the request and handle potential errors
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(NetError::FailedToSendRequest)?;

    // Check if the response status is successful
    if !response.status().is_success() {
        return Err(NetError::ServerError(format!(
            "Server returned status code: {}",
            response.status()
        )));
    }

    // Read the response body and handle potential errors
    let response_text = response
        .text()
        .await
        .map_err(NetError::FailedToReadResponse)?;

    // Deserialize the response and handle potential errors
    let game: Game = serde_json::from_str(&response_text)?;

    Ok(game)
}

pub async fn join_game(game_id: Uuid) -> Result<Player, NetError> {
    let url = format!("{}/game/{}/join", SERVER_ADDR, game_id);
    let client = Client::new();

    // Send the request and handle potential errors
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(NetError::FailedToSendRequest)?;

    // Check if the response status is successful
    if !response.status().is_success() {
        return Err(NetError::ServerError(format!(
            "Server returned status code: {}",
            response.status()
        )));
    }

    // Read the response body and handle potential errors
    let response_text = response
        .text()
        .await
        .map_err(NetError::FailedToReadResponse)?;

    // Deserialize the response and handle potential errors
    let player: Player = serde_json::from_str(&response_text)?;

    Ok(player)
}
