use crate::client_state_mgr::ClientsStatesMgr;
use crate::csv_processor::TransactionLoader;
use crate::transaction_mgr::TransactionMgr;
use crate::{TransactionDetails, TransactionType};

/// Processor to apply new transaction actions
pub struct TransactionsProcessor<'a, L: TransactionLoader> {
    client_state_mgr: &'a mut ClientsStatesMgr,
    transaction_mgr: &'a mut TransactionMgr,
    transaction_loader: L,
}

impl<'a, L: TransactionLoader> TransactionsProcessor<'a, L> {
    pub fn new(
        client_state_mgr: &'a mut ClientsStatesMgr,
        transaction_mgr: &'a mut TransactionMgr,
        transaction_loader: L,
    ) -> Self {
        Self {
            client_state_mgr,
            transaction_mgr,
            transaction_loader,
        }
    }

    /// Apply transaction actions on existing states
    pub fn apply_transaction_actions(&mut self) {
        while let Some(action_details) = self.transaction_loader.next_transaction() {
            match action_details.transaction_type {
                TransactionType::Deposit => self.apply_deposit(action_details),
                TransactionType::Withdrawal => self.apply_withdrawal(action_details),
                TransactionType::Dispute => self.apply_dispute(action_details),
                TransactionType::Resolve => self.apply_resolve(action_details),
                TransactionType::Chargeback => self.apply_chargeback(action_details),
                TransactionType::Unknown => true,
            };
        }
    }

    fn apply_deposit(&mut self, action_details: TransactionDetails) -> bool {
        if action_details.amount.is_none() {
            return false;
        }

        let amount = action_details.amount.unwrap();
        if amount <= 0_f32 {
            return false;
        } else if self.transaction_mgr.transaction_exist(action_details.tx) {
            return false;
        }

        if !self
            .client_state_mgr
            .apply_deposit(action_details.client, amount)
        {
            return false;
        }

        self.transaction_mgr.insert_new_transaction(action_details)
    }

    fn apply_withdrawal(&mut self, action_details: TransactionDetails) -> bool {
        if action_details.amount.is_none() {
            return false;
        } else if self.transaction_mgr.transaction_exist(action_details.tx) {
            return false;
        }

        let amount = action_details.amount.unwrap();
        if amount <= 0_f32 {
            return false;
        }

        if !self
            .client_state_mgr
            .apply_withdrawal(action_details.client, amount)
        {
            return false;
        }

        self.transaction_mgr.insert_new_transaction(action_details)
    }

    fn apply_dispute(&mut self, action_details: TransactionDetails) -> bool {
        if action_details.amount.is_some() {
            return false;
        }

        let transaction = self.transaction_mgr.get_transaction(action_details.tx);
        // If transaction is not found - ignore!
        if transaction.is_none() {
            return false;
        }

        let amount = transaction.unwrap().amount.unwrap();

        if !self
            .client_state_mgr
            .apply_dispute(action_details.client, amount)
        {
            return false;
        }

        true
    }

    fn apply_resolve(&mut self, action_details: TransactionDetails) -> bool {
        if action_details.amount.is_some() {
            return false;
        }

        let transaction = self.transaction_mgr.get_transaction(action_details.tx);
        // If transaction is not found - ignore!
        if transaction.is_none() {
            return false;
        }

        let amount = transaction.unwrap().amount.unwrap();

        if !self
            .client_state_mgr
            .apply_resolve(action_details.client, amount)
        {
            return false;
        }

        true
    }

    fn apply_chargeback(&mut self, action_details: TransactionDetails) -> bool {
        if action_details.amount.is_some() {
            return false;
        }

        let transaction = self.transaction_mgr.get_transaction(action_details.tx);
        // If transaction is not found - ignore!
        if transaction.is_none() {
            return false;
        }

        let amount = transaction.unwrap().amount.unwrap();

        if !self
            .client_state_mgr
            .apply_chargeback(action_details.client, amount)
        {
            return false;
        }

        true
    }
}
