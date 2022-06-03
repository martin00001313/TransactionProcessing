use crate::{TransactionDetails, TransactionType};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// Base transaction manager to keep track on transaction history
pub struct TransactionMgr {
    /// Transaction id to details mapping
    id_to_details: HashMap<u32, TransactionDetails>,
}

impl TransactionMgr {
    /// Create transaction manager
    pub fn new() -> Self {
        Self {
            id_to_details: Default::default(),
        }
    }

    /// Insert new transaction with the specified details
    /// Only deposit and withdrawal transactions should be kept
    /// Each transaction must have a valid amount
    pub fn insert_new_transaction(&mut self, transaction: TransactionDetails) -> bool {
        if transaction.transaction_type != TransactionType::Deposit
            && transaction.transaction_type != TransactionType::Withdrawal
        {
            return false;
        } else if transaction.amount.filter(|d| d >= &0.).is_none() {
            return false;
        }

        match self.id_to_details.entry(transaction.tx) {
            Entry::Occupied(_) => return false,
            Entry::Vacant(v) => {
                v.insert(transaction);
                true
            }
        }
    }

    /// Get transaction by id and client id
    pub fn get_transaction(&self, id: u32, client_id: u16) -> Option<&TransactionDetails> {
        self.id_to_details
            .get(&id)
            .filter(|d| d.client == client_id)
    }

    pub fn transaction_exist(&self, id: u32) -> bool {
        self.id_to_details.contains_key(&id)
    }
}

#[cfg(test)]
mod test {
    use crate::{TransactionDetails, TransactionMgr, TransactionType};
    use float_cmp::approx_eq;

    #[test]
    pub fn test_transaction_mgr() {
        let mut mgr = TransactionMgr::new();

        assert!(mgr.id_to_details.is_empty());

        let mut tx = TransactionDetails {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: None,
        };

        assert!(!mgr.insert_new_transaction(tx.clone()), "Amount is none!");
        tx.amount = Some(-1.);
        assert!(
            !mgr.insert_new_transaction(tx.clone()),
            "Amount is negative!"
        );
        assert!(!mgr.transaction_exist(1));
        tx.amount = Some(2.);
        assert!(mgr.insert_new_transaction(tx.clone()));

        tx.amount = Some(3.);
        assert!(
            !mgr.insert_new_transaction(tx.clone()),
            "Transaction with ID present!"
        );
        assert!(
            approx_eq!(
                f32,
                mgr.get_transaction(1, 1).unwrap().amount.unwrap(),
                2.,
                ulps = 4
            ),
            "Amount shouldn't be changed if transaction is present!"
        );

        assert!(mgr.transaction_exist(1));
    }
}
