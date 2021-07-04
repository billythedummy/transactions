//! Struct representing a transaction of the 5 different transaction types

#[cfg(test)]
mod test;

use serde::{self, Deserialize};
use std::convert::{TryFrom, TryInto};

use crate::cents::Cents;
use crate::client::ClientId;
use crate::err::TransactionError;

pub type TransactionId = u32;

#[serde(rename_all = "lowercase")]
#[derive(Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub client: ClientId,
    pub tx: TransactionId,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<Cents>,
}

#[derive(PartialEq, Debug)]
pub enum DisputableTransactionType {
    Deposit,
    Withdrawal,
}

#[derive(Debug)]
pub struct DisputableTransaction {
    pub transaction_type: DisputableTransactionType,
    pub amount: Cents,
}

impl TryFrom<TransactionType> for DisputableTransactionType {
    type Error = TransactionError;

    fn try_from(t: TransactionType) -> Result<Self, Self::Error> {
        match t {
            TransactionType::Deposit => Ok(Self::Deposit),
            TransactionType::Withdrawal => Ok(Self::Withdrawal),
            _ => Err(TransactionError::TransactionIndisputable(t)),
        }
    }
}

impl TryFrom<Transaction> for DisputableTransaction {
    type Error = TransactionError;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        let transaction_type: DisputableTransactionType = tx.transaction_type.try_into()?;
        Ok(DisputableTransaction {
            transaction_type,
            amount: tx.amount.ok_or_else(|| TransactionError::NoAmount)?,
        })
    }
}
