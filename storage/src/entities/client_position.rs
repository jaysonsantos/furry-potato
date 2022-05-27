use transaction::client::ClientPosition;

use crate::implement_storage;

implement_storage!(
    ClientPosition,
    |this: &ClientPosition| format!("client-position-{}", this.client),
    |old: &ClientPosition, new: &ClientPosition| {
        let mut output = old.clone();
        output.total += new.total;
        output.available += new.available;
        output.held += new.held;
        output
    }
);
