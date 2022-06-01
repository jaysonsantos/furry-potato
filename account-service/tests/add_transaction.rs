use std::sync::Once;

use account_service::{errors::Error::Storage, Service, ServiceImpl};
use color_eyre::eyre::WrapErr;
use storage::{errors::Data::TransactionNotFoundForClient, Error::Data};
use tokio::test;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use transaction::{client::ClientPosition, Transaction, TransactionType};

static TRACING: Once = Once::new();

fn get_test_service() -> ServiceImpl {
    ServiceImpl::with_sled().expect("failed to create service")
}

fn get_test_transaction() -> Transaction {
    Transaction {
        transaction_type: TransactionType::Deposit,
        client: 10,
        transaction_id: 2,
        amount: Some(30.into()),
    }
}

#[test]
async fn add_transaction() {
    let service = get_test_service();
    let transaction = get_test_transaction();
    let transaction = service
        .add_transaction(transaction)
        .await
        .expect("failed to save transaction");
    let saved_transaction = service
        .get_transaction(transaction.client, transaction.transaction_id)
        .await
        .expect("failed to get saved transaction");
    assert_eq!(transaction, saved_transaction);
}

#[test]
async fn client_position_incremental_deposits() {
    setup_tracing();
    let service = get_test_service();
    let transaction = get_test_transaction();
    let total = 30;
    let available = 30;
    for i in 1..100 {
        service
            .add_transaction(Transaction {
                transaction_id: i,
                ..transaction.clone()
            })
            .await
            .expect("failed to save transaction");
        let positions = service
            .get_clients_positions()
            .await
            .expect("failed to get clients positions");
        assert_eq!(positions.len(), 1);
        let position = &positions[0];
        let expected = ClientPosition {
            client: 10,
            total: (total * i).into(),
            available: (available * i).into(),
            held: 0.into(),
            locked: false,
        };
        assert_eq!(position, &expected);
    }
}

#[test]
async fn client_position_dispute_and_solve() -> color_eyre::Result<()> {
    setup_tracing();
    let service = get_test_service();
    let transaction = get_test_transaction();

    let expected = ClientPosition {
        client: 10,
        total: 30.into(),
        available: 30.into(),
        held: 0.into(),
        locked: false,
    };

    let transactions = vec![
        transaction.clone(),
        Transaction {
            transaction_type: TransactionType::Dispute,
            ..transaction.clone()
        },
        Transaction {
            transaction_type: TransactionType::Resolve,
            ..transaction.clone()
        },
    ];
    let expectations = vec![
        expected.clone(),
        ClientPosition {
            available: 0.into(),
            held: 30.into(),
            ..expected.clone()
        },
        ClientPosition {
            available: 30.into(),
            held: 0.into(),
            ..expected.clone()
        },
    ];
    for (transaction, expected) in transactions.into_iter().zip(expectations.into_iter()) {
        service.add_transaction(transaction).await?;
        let positions = service.get_clients_positions().await?;
        assert_eq!(positions.len(), 1);
        let position = &positions[0];

        assert_eq!(position, &expected);
    }
    Ok(())
}

fn setup_tracing() {
    TRACING.call_once(|| {
        Registry::default()
            .with(ErrorLayer::default())
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(fmt::layer())
            .try_init()
            .expect("failed to setup tracing");
        color_eyre::install().expect("failed to setup color_eyre")
    });
}

#[test]
async fn client_position_lock_account() {
    setup_tracing();
    let service = get_test_service();
    let transaction = get_test_transaction();

    let expected = ClientPosition {
        client: 10,
        total: 30.into(),
        available: 30.into(),
        held: 0.into(),
        locked: false,
    };

    let transactions = vec![
        transaction.clone(),
        Transaction {
            transaction_type: TransactionType::Dispute,
            ..transaction.clone()
        },
        Transaction {
            transaction_type: TransactionType::Chargeback,
            ..transaction.clone()
        },
    ];
    let expectations_ = vec![
        expected.clone(),
        ClientPosition {
            available: 0.into(),
            held: 30.into(),
            ..expected.clone()
        },
        ClientPosition {
            available: 0.into(),
            held: 0.into(),
            total: 0.into(),
            locked: true,
            ..expected.clone()
        },
    ];
    for (i, (transaction, expected)) in transactions
        .into_iter()
        .zip(expectations_.into_iter())
        .enumerate()
    {
        service
            .add_transaction(transaction)
            .await
            .expect("failed to save transaction");
        let positions = service
            .get_clients_positions()
            .await
            .wrap_err("failed to get clients positions")
            .unwrap();
        assert_eq!(positions.len(), 1);
        let position = &positions[0];

        assert_eq!(
            position,
            &expected,
            "transaction on position {} was not expected",
            i + 1
        );
    }
}

#[test]
async fn duplicated_transaction() {
    let service = get_test_service();
    let transaction = get_test_transaction();
    service
        .add_transaction(transaction)
        .await
        .expect("failed to save transaction");
    let transaction = Transaction {
        client: 999,
        ..get_test_transaction()
    };
    match service.add_transaction(transaction).await {
        Err(Storage(Data(TransactionNotFoundForClient(_)))) => {}
        other => panic!(
            "this should be a duplicated transaction and not {:?}",
            other
        ),
    };
}
