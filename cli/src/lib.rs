use std::sync::Once;

use color_eyre::{eyre::WrapErr, Result};
use futures_util::{pin_mut, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, Registry};
use transaction::Transaction;

static INSTRUMENTATION: Once = Once::new();

pub fn setup_instrumentation() {
    INSTRUMENTATION.call_once(|| {
        Registry::default()
            .with(fmt::layer())
            .with(ErrorLayer::default())
            .try_init()
            .expect("failed to initialize tracing");
        color_eyre::install().expect("failed to setup color_eyre");
    })
}

pub struct Cli {
    account_service: Box<dyn account_service::Service>,
}

impl Cli {
    #[instrument(err)]
    pub fn new() -> Result<Self> {
        setup_instrumentation();
        Ok(Self {
            account_service: Box::new(account_service::ServiceImpl::new()?),
        })
    }

    #[instrument(skip_all, err)]
    pub async fn process_and_print_transactions<I, O>(&self, input: I, output: O) -> Result<()>
    where
        I: AsyncRead + Unpin + Send,
        O: AsyncWrite + Unpin + Send + Sync,
    {
        self.process_transactions(input)
            .await
            .wrap_err("failed to process transactions")?;
        self.print_clients_positions(output)
            .await
            .wrap_err("failed to print clients positions")?;
        Ok(())
    }

    #[instrument(skip_all, err)]
    async fn process_transactions<I>(&self, input: I) -> Result<()>
    where
        I: AsyncRead + Unpin + Send,
    {
        let transactions = Transaction::from_reader(input).await.enumerate();
        pin_mut!(transactions);

        while let Some((i, transaction)) = transactions.next().await {
            let line = i + 2;
            let transaction = transaction
                .wrap_err_with(|| format!("failed to read transaction on line #{}", line))?;
            self.process_transaction(&transaction)
                .await
                .wrap_err_with(|| format!("failed to process transaction on line #{}", line))?;
        }

        Ok(())
    }

    #[instrument(
        fields(
            client = transaction.client,
            transaction_type = % transaction.transaction_type,
            id = transaction.transaction_id,
            amount = % transaction.amount
        ),
        skip_all,
        err,
    )]
    async fn process_transaction(&self, transaction: &Transaction) -> Result<()> {
        self.account_service
            .add_transaction(transaction)
            .await
            .wrap_err("failed process transaction")?;
        Ok(())
    }

    #[instrument(skip_all, err)]
    async fn print_clients_positions<O>(&self, writer: O) -> Result<()>
    where
        O: AsyncWrite + Unpin + Send + Sync,
    {
        let mut writer = csv_async::AsyncWriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .create_serializer(writer);

        // In case of a big amount clients, this would actually have to be a stream
        let positions = self
            .account_service
            .get_clients_positions()
            .await
            .wrap_err("failed to get clients positions")?;

        for position in positions {
            writer
                .serialize(&position)
                .await
                .wrap_err("failed to serialize client position")?;
        }
        Ok(())
    }
}
