use reqwest::Error as ReqwestError;
use rmp_serde::decode::Error as RmpSerdeDecodeError;
use rmp_serde::encode::Error as RmpSerdeEncodeError;
use serde_json::Error as SerdeJsonError;
use std::io::Error as IoError;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TcpError {
    #[error("Failed to send request: {0}")]
    FailedToSendRequest(#[from] ReqwestError),
    #[error("Failed to read response: {0}")]
    FailedToReadResponse(ReqwestError),
    #[error("Failed to deserialize response: {0}")]
    FailedToDeserializeResponse(#[from] SerdeJsonError),
    #[error("Server returned an error: {0}")]
    ServerError(String),
}

#[derive(Debug, Error)]
pub enum UdpError {
    #[error("IO error: {0}")]
    Io(#[from] IoError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] RmpSerdeEncodeError),
    #[error("MessagePack deserialization error: {0}")]
    MsgPackDeserialization(#[from] RmpSerdeDecodeError),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("Invalid source")]
    InvalidSource,
}
