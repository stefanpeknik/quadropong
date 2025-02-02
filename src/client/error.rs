use std::sync::PoisonError;

use super::net::error::{TcpError, UdpError};
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetError(String),
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Sync error: {0}")]
    SyncError(String),
}

impl From<TcpError> for ClientError {
    fn from(error: TcpError) -> Self {
        ClientError::NetError(format!("TcpError: {}", error))
    }
}

impl From<UdpError> for ClientError {
    fn from(error: UdpError) -> Self {
        ClientError::NetError(format!("UdpError: {}", error))
    }
}

impl From<JoinError> for ClientError {
    fn from(error: JoinError) -> Self {
        ClientError::SyncError(format!("JoinError: {}", error))
    }
}

impl<T> From<PoisonError<T>> for ClientError {
    fn from(error: PoisonError<T>) -> Self {
        ClientError::SyncError(format!("PoisonError: {}", error))
    }
}
