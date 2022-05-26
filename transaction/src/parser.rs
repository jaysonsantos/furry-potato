use std::fmt::Display;

use async_stream::stream;
use csv_async::{AsyncReaderBuilder, Trim};
use enum_display_derive::Display;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;
use tokio_stream::Stream;

use crate::{client::Client, errors::Result};

#[derive(Debug, Deserialize, Serialize, Display, PartialEq, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    #[default]
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Serialize, Display, Default)]
pub enum TransactionState {
    #[default]
    Available,
    Held,
    Locked,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: Client,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
}

impl Transaction {
    pub async fn from_reader<R>(reader: R) -> impl Stream<Item = Result<Transaction>>
    where
        R: AsyncRead + Unpin + Send,
    {
        let mut deserializer = AsyncReaderBuilder::new()
            .delimiter(b',')
            .trim(Trim::All)
            .create_deserializer(reader);
        stream! {
            let stream = deserializer.deserialize::<Transaction>();
            for await result in stream {
                yield result.map_err(|e| e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tokio::test;
    use tokio_stream::StreamExt;

    use crate::{errors::Result, parser::Transaction};

    #[test]
    async fn parse_transactions() {
        let transactions = include_str!("../../fixtures/input-001.csv");
        let cursor = Cursor::new(transactions.as_bytes());
        let transactions: Result<Vec<Transaction>> =
            Transaction::from_reader(cursor).await.collect().await;

        let transactions = transactions.expect("should have transactions");

        assert_eq!(transactions.len(), 5);
    }
}
