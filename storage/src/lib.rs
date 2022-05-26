use async_trait::async_trait;

pub use crate::errors::Error;

pub mod errors;
pub mod sled;

#[async_trait]
/// Storage is used to abstract away databases quirks.
pub trait Storage: Sync {
    fn name(&self) -> &str;
}
