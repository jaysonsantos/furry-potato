use transaction::Transaction;

use crate::implement_storage;

implement_storage!(Transaction, |this: &Transaction| format!(
    "client-transaction-{}-transaction-{}",
    this.client, this.transaction_id
));
