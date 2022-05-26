use account_service::{Service, ServiceImpl};
use tokio::test;
use transaction::{client::ClientPosition, Transaction};

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

fn get_test_service() -> ServiceImpl {
    ServiceImpl::with_sled().expect("failed to create service")
}

#[test]
async fn client_position() {
    let service = get_test_service();
    let transaction = get_test_transaction();
    let total = 30;
    let available = 30;
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
        total: total.into(),
        available: available.into(),
        held: 0.into(),
        locked: false,
    };
    assert_eq!(position, &expected);
}

fn get_test_transaction() -> Transaction {
    Transaction {
        transaction_type: Default::default(),
        client: 10,
        transaction_id: 2,
        amount: 30.into(),
    }
}
