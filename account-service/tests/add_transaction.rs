use account_service::{Service, ServiceImpl};
use tokio::test;
use transaction::{client::ClientPosition, Transaction, TransactionType};

fn get_test_service() -> ServiceImpl {
    ServiceImpl::with_sled().expect("failed to create service")
}

fn get_test_transaction() -> Transaction {
    Transaction {
        transaction_type: Default::default(),
        client: 10,
        transaction_id: 2,
        amount: 30.into(),
    }
}

#[test]
async fn add_transaction() {
    let service = get_test_service();
    let transaction = get_test_transaction();
    service
        .add_transaction(&transaction)
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
    let service = get_test_service();
    let transaction = get_test_transaction();
    let total = 30;
    let available = 30;
    for i in 1..100 {
        service
            .add_transaction(&transaction)
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
async fn client_position_dispute_and_solve() {
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
    let expectations_ = vec![
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
    for (transaction, expected) in transactions.iter().zip(expectations_.iter()) {
        service
            .add_transaction(transaction)
            .await
            .expect("failed to save transaction");
        let positions = service
            .get_clients_positions()
            .await
            .expect("failed to get clients positions");
        assert_eq!(positions.len(), 1);
        let position = &positions[0];

        assert_eq!(position, expected);
    }
}
