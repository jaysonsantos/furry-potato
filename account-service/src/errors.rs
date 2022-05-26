use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Error, Default)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] storage::Error),
    #[error("unknown")]
    #[default]
    Unknown,
}
