//! The transaction processing engine

#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::fmt;

use crate::client::{Client, ClientId};
use crate::err::TransactionError;
use crate::transaction::Transaction;

pub struct Engine {
    clients: HashMap<ClientId, Client>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn handle_tx(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let client_id = tx.client;
        let client = self.clients.entry(client_id).or_insert(Client::new());
        client.handle_tx(tx)
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "client,available,held,total,locked")?;
        for (client_id, client) in self.clients.iter() {
            write!(
                f,
                "\n{},{},{},{},{}",
                client_id,
                client.available(),
                client.held(),
                client.total(),
                client.frozen()
            )?;
        }
        Ok(())
    }
}
