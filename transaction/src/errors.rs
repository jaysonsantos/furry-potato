use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse a csv line")]
    ParsingCsv(#[from] csv_async::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
