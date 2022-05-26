use color_eyre::{eyre::WrapErr, Result};
use futures_util::{pin_mut, StreamExt};
use tokio::{
    fs::File,
    io,
    io::{AsyncRead, AsyncWrite},
};
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, Registry};
use transaction::Transaction;

pub struct Cli {}

impl Cli {
    #[instrument(err)]
    pub fn new() -> Result<Self> {
        Registry::default()
            .with(fmt::layer())
            .with(ErrorLayer::default())
            .init();
        color_eyre::install()?;
        Ok(Self {})
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
    client = transaction.client.id,
    transaction_type = % transaction.transaction_type,
    id = transaction.transaction_id,
    amount = % transaction.amount
    ),
    skip_all,
    err,
    )]
    async fn process_transaction(&self, transaction: &Transaction) -> Result<()> {
        let _ = transaction;
        Ok(())
    }

    #[instrument(skip_all, err)]
    async fn print_clients_positions<O>(&self, mut writer: O) -> Result<()>
    where
        O: AsyncWrite + Unpin + Send + Sync,
    {
        let mut output_file = File::open("../fixtures/output-001.csv").await.unwrap();
        io::copy(&mut output_file, &mut writer).await.unwrap();
        Ok(())
    }
}
