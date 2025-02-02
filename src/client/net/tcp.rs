use log::info;
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
            .timeout(std::time::Duration::from_secs(5))
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
            .timeout(std::time::Duration::from_secs(5))
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
            .timeout(std::time::Duration::from_secs(5))
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

    pub async fn add_bot(&self, game_id: Uuid) -> Result<Player, TcpError> {
        let url = format!("{}/game/{}/add_bot", self.server_addr, game_id);
        info!("Sending request to {}", url);

        // Send the request and handle potential errors
        let response = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(5))
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

    pub async fn remove_bot(&self, game_id: Uuid) -> Result<(), TcpError> {
        let url = format!("{}/game/{}/remove_bot", self.server_addr, game_id);
        info!("Sending request to {}", url);

        // Send the request and handle potential errors
        let response = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(5))
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

        Ok(())
    }

    pub async fn play_again(
        &self,
        game_id: Uuid,
        username: Option<String>,
    ) -> Result<Player, TcpError> {
        let url = format!("{}/game/{}/play_again", self.server_addr, game_id);
        let payload_json = serde_json::to_string(&JoinGameRequest { username })?;

        // Send the request and handle potential errors
        let response = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(5))
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
#[cfg(test)]
mod tests {
    use crate::common::models::GameState;

    use super::*;
    use mockito::Server;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_game_success() {
        let mut server = Server::new_async().await;
        let expected_id = Uuid::new_v4();
        let mock = server
            .mock("POST", "/game")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": expected_id,
                    "players": {},
                    "state": "WaitingForPlayers",
                    "created_at": "2023-10-01T12:34:56Z",
                    "started_at": null,
                    "ball": null,
                    "last_goal_at": null
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.create_game().await;

        mock.assert();
        let game = result.unwrap();
        assert_eq!(game.id, expected_id);
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[tokio::test]
    async fn test_create_game_server_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/game")
            .with_status(500)
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.create_game().await;

        mock.assert();
        assert!(matches!(result, Err(TcpError::ServerError(_))));
    }

    #[tokio::test]
    async fn test_create_game_invalid_response() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/game")
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.create_game().await;

        mock.assert();
        assert!(matches!(
            result,
            Err(TcpError::FailedToDeserializeResponse(_))
        ));
    }

    #[tokio::test]
    async fn test_get_game_success() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let mock = server
            .mock("GET", format!("/game/{}", game_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": game_id,
                    "players": {},
                    "state": "Active",
                    "created_at": "2023-10-01T12:34:56Z",
                    "started_at": "2023-10-01T12:35:00Z",
                    "ball": null,
                    "last_goal_at": null
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.get_game(game_id).await;

        mock.assert();
        let game = result.unwrap();
        assert_eq!(game.id, game_id);
        assert_eq!(game.state, GameState::Active);
    }

    #[tokio::test]
    async fn test_get_game_not_found() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let mock = server
            .mock("GET", format!("/game/{}", game_id).as_str())
            .with_status(404)
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.get_game(game_id).await;

        mock.assert();
        assert!(matches!(result, Err(TcpError::ServerError(_))));
    }

    #[tokio::test]
    async fn test_join_game_with_username() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let username = "test_user";
        let mock = server
            .mock("POST", format!("/game/{}/join", game_id).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(json!({ "username": username }).to_string().as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": player_id,
                    "name": username,
                    "joined_at": "2023-10-01T12:34:56Z",
                    "ping_timestamp": null,
                    "score": 0,
                    "addr": null,
                    "position": "Top",
                    "paddle_position": 0.5,
                    "paddle_delta": 0.0,
                    "paddle_width": 0.2,
                    "is_ready": false,
                    "is_ai": false
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.join_game(game_id, Some(username.to_string())).await;

        mock.assert();
        let player = result.unwrap();
        assert_eq!(player.id, player_id);
        assert_eq!(player.name, username);
    }

    #[tokio::test]
    async fn test_join_game_without_username() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let mock = server
            .mock("POST", format!("/game/{}/join", game_id).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(json!({ "username": null }).to_string().as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": player_id,
                    "name": "",
                    "joined_at": "2023-10-01T12:34:56Z",
                    "ping_timestamp": null,
                    "score": 0,
                    "addr": null,
                    "position": "Bottom",
                    "paddle_position": 0.5,
                    "paddle_delta": 0.0,
                    "paddle_width": 0.2,
                    "is_ready": false,
                    "is_ai": false
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.join_game(game_id, None).await;

        mock.assert();
        let player = result.unwrap();
        assert_eq!(player.id, player_id);
        assert!(player.name.is_empty());
    }

    #[tokio::test]
    async fn test_add_bot_success() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let bot_id = Uuid::new_v4();
        let mock = server
            .mock("POST", format!("/game/{}/add_bot", game_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": bot_id,
                    "name": "AI Bot",
                    "joined_at": "2023-10-01T12:34:56Z",
                    "ping_timestamp": null,
                    "score": 0,
                    "addr": null,
                    "position": "Left",
                    "paddle_position": 0.5,
                    "paddle_delta": 0.0,
                    "paddle_width": 0.2,
                    "is_ready": true,
                    "is_ai": true
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.add_bot(game_id).await;

        mock.assert();
        let player = result.unwrap();
        assert!(player.is_ai);
        assert_eq!(player.id, bot_id);
    }

    #[tokio::test]
    async fn test_add_bot_failure() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let mock = server
            .mock("POST", format!("/game/{}/add_bot", game_id).as_str())
            .with_status(400)
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.add_bot(game_id).await;

        mock.assert();
        assert!(matches!(result, Err(TcpError::ServerError(_))));
    }

    #[tokio::test]
    async fn test_remove_bot_success() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let mock = server
            .mock("POST", format!("/game/{}/remove_bot", game_id).as_str())
            .with_status(200)
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.remove_bot(game_id).await;

        mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_bot_failure() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let mock = server
            .mock("POST", format!("/game/{}/remove_bot", game_id).as_str())
            .with_status(400)
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.remove_bot(game_id).await;

        mock.assert();
        assert!(matches!(result, Err(TcpError::ServerError(_))));
    }

    #[tokio::test]
    async fn test_play_again_success() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let username = "test_user";
        let mock = server
            .mock("POST", format!("/game/{}/play_again", game_id).as_str())
            .match_header("Content-Type", "application/json")
            .match_body(json!({ "username": username }).to_string().as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": player_id,
                    "name": username,
                    "joined_at": "2023-10-01T12:34:56Z",
                    "ping_timestamp": null,
                    "score": 0,
                    "addr": null,
                    "position": "Top",
                    "paddle_position": 0.5,
                    "paddle_delta": 0.0,
                    "paddle_width": 0.2,
                    "is_ready": false,
                    "is_ai": false
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.play_again(game_id, Some(username.to_string())).await;

        mock.assert();
        let player = result.unwrap();
        assert_eq!(player.id, player_id);
        assert_eq!(player.name, username);
    }

    #[tokio::test]
    async fn test_play_again_failure() {
        let mut server = Server::new_async().await;
        let game_id = Uuid::new_v4();
        let mock = server
            .mock("POST", format!("/game/{}/play_again", game_id).as_str())
            .with_status(400)
            .create_async()
            .await;

        let client = TcpClient::new(&server.url());
        let result = client.play_again(game_id, None).await;

        mock.assert();
        assert!(matches!(result, Err(TcpError::ServerError(_))));
    }
}
