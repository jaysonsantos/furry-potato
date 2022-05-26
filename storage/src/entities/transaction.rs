use transaction::Transaction;

use crate::implement_storage;

implement_storage!(
    Transaction,
    |this: &Transaction| format!(
        "client-transaction-{}-transaction-{}",
        this.client, this.transaction_id
    ),
    |old: &Transaction, new: &Transaction| {
        let old = old.clone();
        Transaction {
            transaction_type: new.transaction_type.clone(),
            ..old
        }
    }
);
