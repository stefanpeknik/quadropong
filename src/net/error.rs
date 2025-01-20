use thiserror::Error;

#[derive(Debug, Error)]
pub enum TcpError {
    #[error("Failed to send request: {0}")]
    FailedToSendRequest(#[from] reqwest::Error),
    #[error("Failed to read response: {0}")]
    FailedToReadResponse(reqwest::Error),
    #[error("Failed to deserialize response: {0}")]
    FailedToDeserializeResponse(#[from] serde_json::Error),
    #[error("Server returned an error: {0}")]
    ServerError(String),
}

#[derive(Debug, Error)]
pub enum UdpError {}
