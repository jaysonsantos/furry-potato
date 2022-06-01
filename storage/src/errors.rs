use std::result;

use sled::CompareAndSwapError;
use thiserror::Error;
use transaction::client::Client;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OpeningStorage(#[from] OpeningStorage),
    #[error(transparent)]
    Data(#[from] Data),
}

#[derive(Error, Debug)]
#[error("failed to open storage: {message}")]
pub struct OpeningStorage {
    pub message: String,
    #[source]
    pub source: sled::Error,
}

#[derive(Error, Debug)]
pub enum Data {
    #[error("{0}")]
    Sled(String, #[source] sled::Error),
    #[error("conflicting while updating data")]
    Conflict(#[from] CompareAndSwapError),
    #[error("failed to serialize data")]
    Serialization(#[from] serde_json::Error),
    #[error("key not found {0}")]
    KeyNotFound(String),
    #[error("transaction not found for client {0}")]
    TransactionNotFoundForClient(Client),
    #[error("transaction cannot transition from {0} to {1}")]
    InvalidTransition(String, String),
}
