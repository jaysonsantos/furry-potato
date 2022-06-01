use transaction::Transaction;

use crate::implement_storage;

implement_storage!(
    Transaction,
    |this: &Transaction| format!("transaction-{}", this.transaction_id),
    |this: &Transaction| this.transaction_id
);
