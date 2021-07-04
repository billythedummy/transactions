# Transactions

A toy transactions processing engine that takes a CSV of transactions in chronological order as input and outputs the final state of all accounts involved. 

## Design

### Architecture

The core processing engine is the `engine` module. It maintains a HashMap of `ClientId` to `Client` structs. Each `Client` struct maintains the state of a client's account, keeping track of available funds, held funds, whether the account is currently locked/frozen and which transactions are currently under dispute, and updates its state with transactions that affects it. The engine reads transactions from the input CSV one by one and dispatch them to the respective `Client` for each. 

### Error Handling

Given the importance of getting financial transactions correct, this program errs on the safe side and panics whenever any unrecoverable error or ambiguity occurs, no matter how minor. Most prominent of which is overflows from addition. 

However, the following errors are recoverable, and transactions with these errors are simply ignored:
- Insufficient balance for withdrawal
- Creating a transaction with duplicate transaction IDs
- Disputing/Resolving/Chargeback a transaction that doesn't exist at that point in time or one that is not under dispute

The `TransactionError` type in `err.rs` represents such a recoverable error. 

### Asset Handling

The engine assumes that the asset being transacted has up to four places past the decimal precision, and that the smallest atomic unit of the asset is 0.0001, called a `Cent`. Internally, `Cent`s are represented as `u64`, so no negative values are allowed and the maximum possible amount in one account is `(2^64-1) * 0.0001`, which is around 1.85 quadrillion, enough for most currencies and financial instruments (except for say, Zimbabwean Dollar). This representation was chosen over floating point to avoid rounding errors. 

### Displaying Asset Values

All asset values are displayed to their full 4 decimal places precision - an asset value of 1 will be output as `1.0000`

### Transactions

5 types of transactions:
- `Withdrawal`
- `Deposit`
- `Dispute`
- `Resolve`
- `Chargeback`

### Frozen Accounts

Frozen accounts can still deposit but cannot withdraw. This means they cannot process `Withdrawal` transactions nor `Chargeback` transactions for deposit disputes. 

#### Disputes

Of the 5 transaction types, only `Withdrawal` and `Deposit` transactions can be disputed. They are converted into a separate type `DisputableTransaction` for distinguishing between the other 3 transaction types and for more space-efficient storage within the `Client`'s state.  

A `Deposit` dispute results in funds being transferred from available funds to held funds. The held funds are either returned back to available funds in the case of a `Resolve` or debited in the case of a `Chargeback`. A deposit dispute does nothing if there are not enough available funds. 

A `Withdrawal` dispute results in funds being credited to held funds. The held funds are either debited in the case of a `Resolve` or transferred to available funds in the case of a `Chargeback`. 

Any `Chargeback`s results in an account being frozen. Once a `Chargeback` occurs, the transaction that was charged back can no longer be disputed.

### Input Checking

All input CSV files are assumed to be valid CSVs with a `type, client, tx, amount` header and each row in that specific order. `Dispute`, `Resolve`, `Chargeback` transactions should have the last entry `amount` as either an empty string or pure white space, e.g. `dispute,1,1,`. 

## Tests

### Unit Tests

Unit tests are located in the `test.rs` file/module of each of the modules below:

- `cents`. Check the correctness of the custom deserialization behaviour of 4 decimal places numbers into `u64`. 
- `client`. Runs sequences of test transactions on a single `Client` to check for edge cases and correct behaviour under invalid transactions.
- `engine`. Runs sequences of test transactions and checks the final output CSV generation.
- `transactions`. Checks the type conversion behaviour from `Transaction` to `DisputableTransaction` and deserialization behaviour from CSV.

### Integration Tests

The integration test in `tests/main.rs` runs the cli tool on csv files in `tests/input` and checks the output against the same-named csv file in `tests/output`. 