use std::str::Utf8Error;
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
pub enum UdpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),
    #[error("MessagePack deserialization error: {0}")]
    MsgPackDeserialization(#[from] rmp_serde::decode::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("Invalid source")]
    InvalidSource,
}
