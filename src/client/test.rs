use crate::cents::Cents;
use crate::transaction::{Transaction, TransactionType};

use super::*;

fn run_transactions(
    client: &mut Client,
    txs: Vec<Transaction>,
    expected_avail: Cents,
    expected_held: Cents,
    expected_frozen: bool,
) {
    for tx in txs {
        let _ = client.handle_tx(tx);
    }
    assert_eq!(expected_avail, client.available);
    assert_eq!(expected_held, client.held);
    assert_eq!(expected_frozen, client.frozen);
}

#[test]
fn basic() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 0,
                amount: Some(Cents::new(8)),
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(7)),
            },
        ],
        Cents::new(1),
        Cents::new(0),
        false,
    );
}

#[test]
fn resolve() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(72)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Resolve,
                client: 0,
                tx: 1,
                amount: None,
            },
        ],
        Cents::new(72),
        Cents::new(0),
        false,
    );

    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 2,
                amount: Some(Cents::new(70)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 2,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Resolve,
                client: 0,
                tx: 2,
                amount: None,
            },
        ],
        Cents::new(2),
        Cents::new(0),
        false,
    );
}

#[test]
fn duplicate_tx_id() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 0,
                amount: Some(Cents::new(34)),
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 0,
                amount: Some(Cents::new(33)),
            },
        ],
        Cents::new(34),
        Cents::new(0),
        false,
    );
}

#[test]
fn overdraft() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 0,
                amount: Some(Cents::new(3536)),
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(3537)),
            },
        ],
        Cents::new(3536),
        Cents::new(0),
        false,
    );
}

#[test]
fn no_withdrawal_after_frozen() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(3242)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Chargeback,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 2,
                amount: Some(Cents::new(1)),
            },
            // try to withdraw money via deposit dispute chargeback
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 3,
                amount: Some(Cents::new(169)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 3,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Chargeback,
                client: 0,
                tx: 3,
                amount: None,
            },
        ],
        Cents::new(0),
        Cents::new(169),
        true,
    );
}

#[test]
fn no_chargeback_twice() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 435,
                amount: Some(Cents::new(100)),
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(99)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Chargeback,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Chargeback,
                client: 0,
                tx: 1,
                amount: None,
            },
        ],
        Cents::new(100),
        Cents::new(0),
        true,
    );
}

#[test]
fn no_dispute_twice() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(69)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
        ],
        Cents::new(0),
        Cents::new(69),
        false,
    );
}

#[test]
fn not_enough_to_dispute_deposit() {
    let mut client = Client::new();
    run_transactions(
        &mut client,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(69)),
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 2,
                amount: Some(Cents::new(1)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 1,
                amount: None,
            },
        ],
        Cents::new(68),
        Cents::new(0),
        false,
    );
}
