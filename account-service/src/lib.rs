use std::{
    fmt::{Debug, Formatter},
    result,
};

use async_trait::async_trait;
use futures::{pin_mut, StreamExt};
use rust_decimal::Decimal;
use storage::sled::Sled;
use tracing::instrument;
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
        self.storage
            .create_or_update(&client_position, Self::merge_client_position)?;
        Ok(())
    }

    fn merge_transaction(
        old: &Transaction,
        new: &Transaction,
    ) -> result::Result<Transaction, storage::errors::Data> {
        if !Self::can_transition(old, new) {
            return Err(storage::errors::Data::InvalidTransition(
                old.transaction_type.to_string(),
                new.transaction_type.to_string(),
            ));
        }
        let old = old.clone();
        Ok(Transaction {
            transaction_type: new.transaction_type.clone(),
            ..old
        })
    }

    #[instrument(fields(old = %old.transaction_type, new = %new.transaction_type))]
    fn can_transition(old: &Transaction, new: &Transaction) -> bool {
        match old.transaction_type {
            TransactionType::Deposit => new.transaction_type == TransactionType::Dispute,
            TransactionType::Withdrawal => false,
            TransactionType::Dispute => [TransactionType::Resolve, TransactionType::Chargeback]
                .contains(&new.transaction_type),
            TransactionType::Resolve => false,
            TransactionType::Chargeback => false,
        }
    }

    fn merge_client_position(
        old: &ClientPosition,
        new: &ClientPosition,
    ) -> result::Result<ClientPosition, storage::errors::Data> {
        let mut output = old.clone();
        output.total += new.total;
        output.available += new.available;
        output.held += new.held;
        output.locked = new.locked;
        Ok(output)
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
    #[instrument]
    async fn add_transaction(&self, transaction: &Transaction) -> Result<()> {
        self.storage
            .create_or_update(transaction, Self::merge_transaction)?;
        self.update_client_position(transaction).await?;
        Ok(())
    }

    #[instrument]
    async fn get_transaction(&self, client: Client, transaction_id: u32) -> Result<Transaction> {
        let entity = self.storage.get(&Transaction {
            client,
            transaction_id,
            ..Default::default()
        })?;
        Ok(entity)
    }

    #[instrument]
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
