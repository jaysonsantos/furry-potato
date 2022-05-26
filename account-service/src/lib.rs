use std::fmt::Debug;

use async_trait::async_trait;
use rust_decimal::Decimal;
use transaction::{
    client::{Client, ClientPosition},
    Transaction,
};

use crate::errors::Result;

pub mod errors;

#[async_trait]
/// Storage is just an abstraction of what would be a database.
pub trait Service: Debug {
    fn name(&self) -> &str;
    async fn add_transaction(&self, transaction: &Transaction) -> Result<()>;
    async fn get_transaction(&self, client: &Client, transaction_id: u32) -> Result<Transaction>;
    async fn get_clients_positions(&self) -> Result<Vec<ClientPosition>>;
    async fn update_client_position(&self, client: &Client, operation: Operation) -> Result<()>;
}

/// Operation is used to mimic atomic operations on a database for example.
pub struct Operation {
    pub total: Decimal,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: Decimal,
}
