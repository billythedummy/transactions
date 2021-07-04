use csv::StringRecord;

use crate::cents::Cents;
use crate::err::TransactionError;

use super::*;

fn check_into_disputable(amount: Cents, tt: TransactionType) {
    let tx = Transaction {
        transaction_type: tt,
        client: 0,
        tx: 0,
        amount: Some(amount),
    };
    let disputable: DisputableTransaction = tx.try_into().unwrap();
    let expected_type = match tt {
        TransactionType::Withdrawal => DisputableTransactionType::Withdrawal,
        TransactionType::Deposit => DisputableTransactionType::Deposit,
        _ => panic!("unreachable"),
    };
    assert_eq!(expected_type, disputable.transaction_type);
    assert_eq!(amount, disputable.amount);
}

#[test]
fn into_disputable() {
    check_into_disputable(Cents::new(69), TransactionType::Withdrawal);
    check_into_disputable(Cents::new(23433242), TransactionType::Deposit);
}

fn check_indisputable(tt: TransactionType) {
    let tx = Transaction {
        transaction_type: tt,
        client: 0,
        tx: 0,
        amount: Some(Cents::new(1)),
    };
    let res: Result<DisputableTransaction, TransactionError> = tx.try_into();
    assert_eq!(
        TransactionError::TransactionIndisputable(tt),
        res.unwrap_err()
    );
}

#[test]
fn indisputable() {
    check_indisputable(TransactionType::Dispute);
    check_indisputable(TransactionType::Chargeback);
    check_indisputable(TransactionType::Resolve);
}

fn check_malformed_disputable(tt: TransactionType) {
    let tx = Transaction {
        transaction_type: tt,
        client: 0,
        tx: 0,
        amount: None,
    };
    let res: Result<DisputableTransaction, TransactionError> = tx.try_into();
    assert_eq!(TransactionError::NoAmount, res.unwrap_err());
}

#[test]
fn malformed_disputable() {
    check_malformed_disputable(TransactionType::Withdrawal);
    check_malformed_disputable(TransactionType::Deposit);
}

fn check_deser(row: Vec<&str>, expected: Transaction) {
    let record = StringRecord::from(row);
    let tx: Transaction = record.deserialize(None).unwrap();
    assert_eq!(expected, tx);
}

#[test]
fn deser() {
    check_deser(
        vec!["withdrawal", "0", "1", "1.0234"],
        Transaction {
            transaction_type: TransactionType::Withdrawal,
            client: 0,
            tx: 1,
            amount: Some(Cents::new(10234)),
        },
    );
    check_deser(
        vec!["deposit", "12314", "3444454514", "0.0210"],
        Transaction {
            transaction_type: TransactionType::Deposit,
            client: 12314,
            tx: 3444454514,
            amount: Some(Cents::new(210)),
        },
    );
    check_deser(
        vec!["dispute", "23414", "459567213", ""],
        Transaction {
            transaction_type: TransactionType::Dispute,
            client: 23414,
            tx: 459567213,
            amount: None,
        },
    );
    check_deser(
        vec!["resolve", "1243", "2322", ""],
        Transaction {
            transaction_type: TransactionType::Resolve,
            client: 1243,
            tx: 2322,
            amount: None,
        },
    );
    check_deser(
        vec!["chargeback", "5245", "3453", ""],
        Transaction {
            transaction_type: TransactionType::Chargeback,
            client: 5245,
            tx: 3453,
            amount: None,
        },
    );
}
