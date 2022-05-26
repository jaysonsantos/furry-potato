use std::result;

use crate::errors::Data;
pub use crate::errors::{Error, Result};

pub mod entities;
pub mod errors;
pub mod sled;

pub trait ToStorage {
    fn to_bytes(&self) -> Vec<u8>;
    fn get_updated(&self, new: &Self) -> Self;
}

pub trait FromStorage {
    fn from_bytes(input: &[u8]) -> result::Result<Self, Data>
    where
        Self: Sized;
}

pub trait ToFromStorage: ToStorage + FromStorage + Send + Sync {
    fn partition(&self) -> usize;
    fn primary_key(&self) -> String;
}
