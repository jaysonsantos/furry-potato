use std::result;

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    OpeningStorage(#[from] OpeningStorage),
}

#[derive(Error, Debug)]
#[error("failed to open storage: {message}")]
pub struct OpeningStorage {
    pub message: String,
    #[source]
    pub source: sled::Error,
}
