use crate::cents::Cents;
use crate::transaction::{Transaction, TransactionType};

use super::Engine;

fn run_transactions(engine: &mut Engine, txs: Vec<Transaction>, expected_clients: &str) {
    for tx in txs {
        let _ = engine.handle_tx(tx);
    }
    let mut expected_itr = expected_clients.lines();
    let out = format!("{}", engine);
    let mut out_itr = out.lines();
    out_itr.next();
    loop {
        let expected_line = expected_itr.next();
        let out_line = out_itr.next();
        if expected_line.is_none() && out_line.is_none() {
            return;
        }
        assert_eq!(expected_line.unwrap(), out_line.unwrap());
    }
}

#[test]
fn basic() {
    let mut engine = Engine::new();
    run_transactions(
        &mut engine,
        vec![Transaction {
            transaction_type: TransactionType::Deposit,
            client: 0,
            tx: 0,
            amount: Some(Cents::new(1)),
        }],
        "0,0.0001,0.0000,0.0001,false",
    );
}

#[test]
fn locked() {
    let mut engine = Engine::new();
    run_transactions(
        &mut engine,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 1,
                amount: Some(Cents::new(126929)),
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
        "0,0.0000,0.0000,0.0000,true",
    );
}

#[test]
fn dispute() {
    let mut engine = Engine::new();
    run_transactions(
        &mut engine,
        vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 2,
                tx: 1,
                amount: Some(Cents::new(100)),
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 2,
                tx: 2,
                amount: Some(Cents::new(99)),
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 2,
                tx: 2,
                amount: None,
            },
        ],
        "2,0.0001,0.0099,0.0100,false",
    );
}
