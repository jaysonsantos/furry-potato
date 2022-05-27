use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use futures::{pin_mut, StreamExt};
use rust_decimal::Decimal;
use storage::sled::Sled;
use transaction::{
    client::{Client, ClientPosition},
    Transaction, TransactionType,
};

use crate::errors::Result;

pub mod errors;

#[async_trait]
/// Storage is just an abstraction of what would be a database.
pub trait Service: Debug + Sync {
    async fn add_transaction(&self, transaction: &Transaction) -> Result<()>;
    async fn get_transaction(&self, client: Client, transaction_id: u32) -> Result<Transaction>;
    async fn get_clients_positions(&self) -> Result<Vec<ClientPosition>>;
}

/// Operation is used to mimic atomic operations on a database for example.
pub struct Operation {
    pub total: Decimal,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: Decimal,
}

pub struct ServiceImpl {
    storage: Sled,
}

impl ServiceImpl {
    pub fn with_sled() -> Result<Self> {
        let storage = Sled::new()?;
        Ok(Self { storage })
    }

    /// This mimics atomic operations by using database's ability to do addition/subtraction without
    /// having to fetch the value first, like:
    /// update client set available = available + 30 where client_id = 1
    async fn update_client_position(&self, transaction: &Transaction) -> Result<()> {
        let amount = transaction.amount;
        let mut client_position = ClientPosition {
            client: transaction.client,
            ..Default::default()
        };
        match transaction.transaction_type {
            TransactionType::Deposit => {
                client_position.available = amount;
            }
            TransactionType::Withdrawal => {
                client_position.available = -amount;
            }
            TransactionType::Dispute => {
                client_position.held = amount;
            }
            TransactionType::Resolve => {
                client_position.held = -amount;
            }
            TransactionType::Chargeback => {
                client_position.locked = true;
                client_position.held = -amount;
                client_position.available = -amount;
            }
        };
        client_position.available -= client_position.held;
        client_position.total += client_position.held + client_position.available;
        self.storage.create_or_update(&client_position)?;
        Ok(())
    }
}

impl Debug for ServiceImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ServiceImpl<")?;
        write!(f, "{}", self.storage)?;
        f.write_str(">")
    }
}

#[async_trait]
impl Service for ServiceImpl {
    async fn add_transaction(&self, transaction: &Transaction) -> Result<()> {
        self.storage.create_or_update(transaction)?;
        self.update_client_position(transaction).await?;
        Ok(())
    }

    async fn get_transaction(&self, client: Client, transaction_id: u32) -> Result<Transaction> {
        let entity = self.storage.get(&Transaction {
            client,
            transaction_id,
            ..Default::default()
        })?;
        Ok(entity)
    }

    async fn get_clients_positions(&self) -> Result<Vec<ClientPosition>> {
        let mut output = vec![];
        let list = self
            .storage
            .list::<ClientPosition>("client-position-")
            .await;
        pin_mut!(list);
        while let Some(item) = list.next().await {
            output.push(item?);
        }
        Ok(output)
    }
}
