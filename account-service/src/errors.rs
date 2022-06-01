use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] storage::Error),
    #[error("account locked")]
    AccountLocked,
    #[error("amount cannot be negative")]
    AmountCannotBeNegative,
    #[error("unknown")]
    Unknown,
}
