use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Client {
    #[serde(rename = "client")]
    pub id: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientPosition {
    #[serde(flatten)]
    client: Client,
    total: Decimal,
    available: Decimal,
    held: Decimal,
    locked: Decimal,
}
