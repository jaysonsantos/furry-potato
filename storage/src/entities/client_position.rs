use transaction::client::ClientPosition;

use crate::implement_storage;

implement_storage!(ClientPosition, |this: &ClientPosition| format!(
    "client-position-{}",
    this.client
));
