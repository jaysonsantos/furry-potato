use std::fmt::{Debug, Formatter, Write};

use async_trait::async_trait;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use storage::{sled::Sled, Storage};
use transaction::{
    client::{Client, ClientPosition},
    Transaction,
};

use crate::errors::Result;

pub mod errors;

#[async_trait]
/// Storage is just an abstraction of what would be a database.
pub trait Service: Debug + Sync {
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

pub struct ServiceImpl {
    storage: Box<dyn Storage>,
}

impl ServiceImpl {
    pub fn new() -> Result<Self> {
        Ok(Self {
            storage: Box::new(Sled::new()?),
        })
    }
}

impl Debug for ServiceImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ServiceImpl<")?;
        f.write_str(self.storage.name())?;
        f.write_str(">")
    }
}

#[async_trait]
impl Service for ServiceImpl {
    async fn add_transaction(&self, _transaction: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn get_transaction(&self, _client: &Client, _transaction_id: u32) -> Result<Transaction> {
        Ok(Transaction::default())
    }

    async fn get_clients_positions(&self) -> Result<Vec<ClientPosition>> {
        Ok(vec![
            ClientPosition {
                client: 1,
                total: Decimal::from_f64(1.5).expect("failed to get decimal"),
                available: Decimal::from_f64(0.0).expect("failed to get decimal"),
                held: Decimal::from_f64(1.5).expect("failed to get decimal"),
                locked: false,
            },
            ClientPosition {
                client: 2,
                total: Decimal::from_f64(2.0).expect("failed to get decimal"),
                available: Decimal::from_f64(0.0).expect("failed to get decimal"),
                held: Decimal::from_f64(2.0).expect("failed to get decimal"),
                locked: false,
            },
        ])
    }

    async fn update_client_position(&self, _client: &Client, _operation: Operation) -> Result<()> {
        Ok(())
    }
}
