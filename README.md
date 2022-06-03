# Transaction processing workflow
Application responsible to consume transaction actions, do
processing of data and return client account base results based
on the calculation.


Supported transaction action types:
1. Deposit - credit to the client's asset account
2. Withdrawal - debit to the client's asset account,
3. Dispute - erroneous transaction claim by customer.
4. Resolve - resolution to a dispute.
5. Chargeback - reverse of the transaction.

Note: In case Dispute/Resolve/Chargeback client should match to actual transaction's client.
Otherwise, the action is skipped.

Input data format - CSV file. Properties with proper description - transaction_details.rs

Output is csv content - in std::io. Properties with description - client_state.rs

#The programming language 
Rust (100%)


#Testing
Unit tests - includes coverage of data serialization/deserialization,
client account details update, transaction  state tracking, etc.

An example of input csv file is under /src/test_utils/ - transactions.csv


#How to run the application
1. download the sources
2. "cargo build"
3. "cargo run -- arg1 > arg2", where:
   1. arg1 is csv file of transaction details(mandatory),
   2. arg2 output csv file location for client details. Optional - if not provided, will just print result
   
Example: cargo run -- src/test_utils/transactions.csv > clients_summary.csv

#How to run the unit test for the application
1. download the sources
2. "cargo build"
3. "cargo test"


#Notes of input csv data processing
1. transaction action types(transaction_type), should be lowercase. Data is trimmed before processing
2. 'client' and 'tx' are integers. Before processing the content is trimmed.
3. 'amount' is floating point number, non-mandatory- data processor expects that it should be provided
    for Deposits and Withdrawals


#Notes of transactions state processing
1. In case of 'Deposit' & 'Withdrawal' are checked:
   1. tx -  to keep uniqueness of it. I.e. is there is a transaction with the same ID, the new ones will be ignored
   2. amount should be >= integer. Rows not having or having negative amount will be ignored
2. In case of 'Dispute', 'Resolve' and 'Chargeback' are checked:
   1. tx - to make sure data with the 'tx' value has been processed (to determine the amount). Raw will be ignored otherwise.
   2. amount - should not be provided
   3. does proper check based on type
   

#Points to improve/check
1. If the account is locked, should we consider upcoming actions for the client? 
Now it continues to consider, but can easily be blocked by uncommenting filter in get_client_details
2. CSV data loader is a base trait, which provide a new entity per each iteration.
   So, it will be easy to integrate web streams, large file streams, etc.
3. In case of 'Resolve' and 'Chargeback' the processor only checks correctness of client id, transaction id, etc.
   'Dispute' actions absence is not considered, but it's logically correct to add such restriction,
   i.e. if 'Resolve' and 'Chargeback' action should be after respective 'Dispute'

#Example of non-fully correct behaviors:
1. Multi-dispute of the transaction
   1. apply deposit: client - {client - 1, tx - 1, amount - 10.0} => state after operation: (1, available: 10, held: 0, total: 10, locked: false)
   2. apply deposit: client - {client - 1, tx - 2, amount - 15.0} => state after operation: (1, available: 25, held: 0, total: 25, locked: false)
   3. dispute the transaction - {client - 1, tx - 1} => state after operation: {1, available- 15, held-10, total - 25}
   4. dispute the transaction again - {client - 1, tx - 1} => state after operation: {1, available- 5, held-20, total - 25}
2. Resolve of non disputed transaction
3. Multi-resolve of the same disputed transaction
4. Chargeback of non-disputed transaction
5. Multi-chargeback of disputed transaction
6. Multi-dispute the same transaction w/o resolve or chargeback

All above-mentioned logically incorrect(probably) issue can be easily solved br tracing the last transaction action type for the given transaction ID
and keep last action type to next allowed action types mapping.
   