//! Error types

use thiserror::Error;

use crate::cents::Cents;
use crate::transaction::{TransactionId, TransactionType};

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
    #[error("Insufficient Balance. Available: {available}. Requested withdrawal: {requested}")]
    InsufficientBalance { available: Cents, requested: Cents },

    #[error("Transaction {0} already exists")]
    DuplicateTransaction(TransactionId),

    #[error("Transaction {0} does not exist")]
    TransactionDoesNotExist(TransactionId),

    #[error("Transaction {0} is not under dispute")]
    TransactionNotUnderDispute(TransactionId),

    #[error("Transaction type {0:?} not disputable")]
    TransactionIndisputable(TransactionType),

    #[error("Transaction {0} already charged back")]
    AlreadyChargedBack(TransactionId),

    #[error("No amount specified for transaction")]
    NoAmount,

    #[error("Account frozen")]
    AccountFrozen,
}
