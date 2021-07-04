//! The Client struct, represents the state of the funds of an individual client's account.

#[cfg(test)]
mod test;

use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

use crate::cents::Cents;
use crate::err::TransactionError;
use crate::transaction::{
    DisputableTransaction, DisputableTransactionType, Transaction, TransactionId, TransactionType,
};

pub type ClientId = u16;

pub struct Client {
    available: Cents,
    held: Cents,
    frozen: bool,
    transactions: HashMap<TransactionId, DisputableTransaction>,
    disputes: HashSet<TransactionId>,
    chargebacks: HashSet<TransactionId>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            available: Cents::new(0),
            held: Cents::new(0),
            frozen: false,
            transactions: HashMap::new(),
            disputes: HashSet::new(),
            chargebacks: HashSet::new(),
        }
    }

    pub fn available(&self) -> Cents {
        self.available
    }

    pub fn held(&self) -> Cents {
        self.held
    }

    pub fn total(&self) -> Cents {
        (self.available + self.held).unwrap()
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }

    pub fn handle_tx(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        match tx.transaction_type {
            TransactionType::Deposit => self.handle_deposit(tx),
            TransactionType::Withdrawal => self.handle_withdrawal(tx),
            TransactionType::Dispute => self.handle_dispute(tx),
            TransactionType::Resolve => self.handle_resolve(tx),
            TransactionType::Chargeback => self.handle_chargeback(tx),
        }
    }

    fn handle_deposit(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let id = tx.tx;
        let tx: DisputableTransaction = tx.try_into()?;
        let amount = tx.amount;
        // update below must be atomic
        self.insert_disputable_tx(id, tx)?;
        // unwrap safety: panics if sum overflows
        self.available = (self.available + amount).unwrap();
        Ok(())
    }

    fn handle_withdrawal(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        if self.frozen {
            return Err(TransactionError::AccountFrozen);
        }
        let id = tx.tx;
        let tx: DisputableTransaction = tx.try_into()?;
        let new_available = self.available_after_debit(tx.amount)?;
        // update below must be atomic
        self.insert_disputable_tx(id, tx)?;
        self.available = new_available;
        Ok(())
    }

    fn handle_dispute(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let id = tx.tx;
        let disputed = self.get_disputable_transaction(id)?;
        let amount = disputed.amount;
        match disputed.transaction_type {
            DisputableTransactionType::Deposit => self.handle_deposit_dispute(id, amount),
            DisputableTransactionType::Withdrawal => self.handle_withdrawal_dispute(id, amount),
        }
    }

    fn handle_deposit_dispute(
        &mut self,
        deposit_id: TransactionId,
        amount: Cents,
    ) -> Result<(), TransactionError> {
        let new_available = self.available_after_debit(amount)?;
        self.insert_dispute(deposit_id)?;
        self.available = new_available;
        self.held = (self.held + amount).unwrap();
        Ok(())
    }

    fn handle_withdrawal_dispute(
        &mut self,
        withdrawal_id: TransactionId,
        amount: Cents,
    ) -> Result<(), TransactionError> {
        self.insert_dispute(withdrawal_id)?;
        self.held = (self.held + amount).unwrap();
        Ok(())
    }

    fn handle_resolve(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let id = tx.tx;
        let disputed = self.get_disputable_transaction(id)?;
        let amount = disputed.amount;
        if !self.disputes.contains(&id) {
            return Err(TransactionError::TransactionNotUnderDispute(id));
        }
        match disputed.transaction_type {
            DisputableTransactionType::Deposit => self.handle_deposit_resolve(id, amount),
            DisputableTransactionType::Withdrawal => self.handle_withdrawal_resolve(id, amount),
        }
        Ok(())
    }

    fn handle_deposit_resolve(&mut self, deposit_id: TransactionId, amount: Cents) {
        // unwrap safety: panics if insufficient balance in held, i.e. the engine messed up
        self.held = (self.held - amount).unwrap();
        self.available = (self.available + amount).unwrap();
        self.disputes.remove(&deposit_id);
    }

    fn handle_withdrawal_resolve(&mut self, withdrawal_id: TransactionId, amount: Cents) {
        // unwrap safety: panics if insufficient balance in held, i.e. the engine messed up
        self.held = (self.held - amount).unwrap();
        self.disputes.remove(&withdrawal_id);
    }

    fn handle_chargeback(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let id = tx.tx;
        let disputed = self.get_disputable_transaction(id)?;
        let amount = disputed.amount;
        if !self.disputes.contains(&id) {
            return Err(TransactionError::TransactionNotUnderDispute(id));
        }
        match disputed.transaction_type {
            DisputableTransactionType::Deposit => self.handle_deposit_chargeback(id, amount),
            DisputableTransactionType::Withdrawal => self.handle_withdrawal_chargeback(id, amount),
        }
        self.frozen = true;
        Ok(())
    }

    fn handle_deposit_chargeback(&mut self, deposit_id: TransactionId, amount: Cents) {
        // Frozen accounts are not allowed to withdraw via deposit dispute chargeback
        if self.frozen {
            return;
        }
        // unwrap safety: panics if insufficient balance in held, i.e. the engine messed up
        self.held = (self.held - amount).unwrap();
        self.disputes.remove(&deposit_id);
        self.chargebacks.insert(deposit_id);
    }

    fn handle_withdrawal_chargeback(&mut self, withdrawal_id: TransactionId, amount: Cents) {
        self.available = (self.available + amount).unwrap();
        // unwrap safety: panics if insufficient balance in held, i.e. the engine messed up
        self.held = (self.held - amount).unwrap();
        self.disputes.remove(&withdrawal_id);
        self.chargebacks.insert(withdrawal_id);
    }

    fn insert_disputable_tx(
        &mut self,
        id: TransactionId,
        tx: DisputableTransaction,
    ) -> Result<(), TransactionError> {
        if self.transactions.contains_key(&id) {
            return Err(TransactionError::DuplicateTransaction(id));
        }
        self.transactions.insert(id, tx);
        Ok(())
    }

    fn insert_dispute(&mut self, id: TransactionId) -> Result<(), TransactionError> {
        if self.disputes.contains(&id) {
            return Err(TransactionError::DuplicateTransaction(id));
        }
        if self.chargebacks.contains(&id) {
            return Err(TransactionError::AlreadyChargedBack(id));
        }
        self.disputes.insert(id);
        Ok(())
    }

    /// Calculates, but does not update, the available balance after a debit
    fn available_after_debit(&self, debit: Cents) -> Result<Cents, TransactionError> {
        (self.available - debit).ok_or_else(|| TransactionError::InsufficientBalance {
            available: self.available,
            requested: debit,
        })
    }

    fn get_disputable_transaction(
        &self,
        id: TransactionId,
    ) -> Result<&DisputableTransaction, TransactionError> {
        self.transactions
            .get(&id)
            .ok_or_else(|| TransactionError::TransactionDoesNotExist(id))
    }
}
