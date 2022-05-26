use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type Client = u16;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct ClientPosition {
    pub client: Client,

    #[serde(with = "rust_decimal::serde::str")]
    pub total: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub available: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub held: Decimal,
    pub locked: bool,
}
