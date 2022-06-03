use crate::client_state_mgr::ClientsStatesMgr;
use crate::csv_processor::TransactionLoader;
use crate::transaction_mgr::TransactionMgr;
use crate::{TransactionDetails, TransactionType};

/// Processor to apply new transaction actions
pub struct TransactionsProcessor<'a, L: TransactionLoader> {
    /// Clients state processor
    client_state_mgr: &'a mut ClientsStatesMgr,
    /// Transactions state processor - i.e. created via Deposit & Withdrawal
    transaction_mgr: &'a mut TransactionMgr,
    /// Transaction actions loader/streamer
    transaction_loader: L,
}

impl<'a, L: TransactionLoader> TransactionsProcessor<'a, L> {
    /// Generate base processor based on provided details
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
                TransactionType::Unknown => false,
            };
        }
    }

    fn apply_deposit(&mut self, action_details: TransactionDetails) -> bool {
        if action_details.transaction_type != TransactionType::Deposit
            || action_details.amount.is_none()
        {
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
        if action_details.transaction_type != TransactionType::Withdrawal
            || action_details.amount.is_none()
        {
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
        if action_details.transaction_type != TransactionType::Dispute
            || action_details.amount.is_some()
        {
            return false;
        }

        let transaction = self
            .transaction_mgr
            .get_transaction(action_details.tx, action_details.client);
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
        if action_details.transaction_type != TransactionType::Resolve
            || action_details.amount.is_some()
        {
            return false;
        }

        let transaction = self
            .transaction_mgr
            .get_transaction(action_details.tx, action_details.client);
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
        if action_details.transaction_type != TransactionType::Chargeback
            || action_details.amount.is_some()
        {
            return false;
        }

        let transaction = self
            .transaction_mgr
            .get_transaction(action_details.tx, action_details.client);
        // If transaction is not found - ignore!
        if transaction.is_none() {
            return false;
        } else if transaction.as_ref().unwrap().client != action_details.client {
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

#[cfg(test)]
mod test {
    use crate::csv_processor::TransactionLoader;
    use crate::{
        ClientsStatesMgr, TransactionDetails, TransactionMgr, TransactionType,
        TransactionsProcessor,
    };
    use float_cmp::approx_eq;

    pub struct TransactionTestLoader {
        data: Vec<TransactionDetails>,
        curr_idx: usize,
    }

    impl TransactionLoader for TransactionTestLoader {
        fn next_transaction(&mut self) -> Option<TransactionDetails> {
            match self.data.get(self.curr_idx) {
                Some(data) => {
                    self.curr_idx += 1;
                    Some(data.clone())
                }
                None => None,
            }
        }
    }

    #[test]
    pub fn test_deposit() {
        let loader = TransactionTestLoader {
            data: vec![],
            curr_idx: 0,
        };

        let mut client_mgr = ClientsStatesMgr::new();
        let mut transaction_mgr = TransactionMgr::new();

        let mut mgr = TransactionsProcessor::new(&mut client_mgr, &mut transaction_mgr, loader);

        let mut action = TransactionDetails {
            transaction_type: TransactionType::Deposit,
            client: 2,
            tx: 1,
            amount: None,
        };

        assert!(
            !mgr.apply_deposit(action.clone()),
            "Should be failed as amount is not provided!"
        );
        assert!(mgr.client_state_mgr.get_states().is_empty());
        assert!(!mgr.transaction_mgr.transaction_exist(1));

        action.amount = Some(13.);
        assert!(mgr.apply_deposit(action.clone()));

        let clients = mgr.client_state_mgr.get_states();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].client, 2);
        assert!(approx_eq!(f32, clients[0].available, 13., ulps = 4));
        assert!(approx_eq!(f32, clients[0].held, 0., ulps = 4));
        assert!(approx_eq!(f32, clients[0].total, 13., ulps = 4));
        assert!(!clients[0].locked);

        let transaction = mgr.transaction_mgr.get_transaction(1, 2);
        assert!(transaction.is_some());
        assert_eq!(
            transaction.unwrap().transaction_type,
            TransactionType::Deposit
        );
        assert!(approx_eq!(
            f32,
            transaction.unwrap().amount.unwrap(),
            13.,
            ulps = 4
        ));

        action.amount = Some(23.);
        assert!(
            !mgr.apply_deposit(action.clone()),
            "Transaction ID is not unique!"
        );

        action.tx = 3;
        assert!(
            mgr.apply_deposit(action.clone()),
            "Transaction ID is unique!"
        );

        let clients = mgr.client_state_mgr.get_states();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].client, 2);
        assert!(approx_eq!(f32, clients[0].available, 36., ulps = 4));
        assert!(approx_eq!(f32, clients[0].held, 0., ulps = 4));
        assert!(approx_eq!(f32, clients[0].total, 36., ulps = 4));
        assert!(!clients[0].locked);

        action.client = 4;
        assert!(
            !mgr.apply_deposit(action.clone()),
            "Transaction ID is not unique!"
        );
        assert_eq!(mgr.client_state_mgr.get_states().len(), 1);

        action.tx = 5;
        assert!(
            mgr.apply_deposit(action.clone()),
            "Transaction ID is unique!"
        );

        let clients = mgr.client_state_mgr.get_states();
        assert_eq!(clients.len(), 2);
    }

    #[test]
    pub fn test_withdraw() {
        let loader = TransactionTestLoader {
            data: vec![],
            curr_idx: 0,
        };

        let mut client_mgr = ClientsStatesMgr::new();
        let mut transaction_mgr = TransactionMgr::new();

        let mut mgr = TransactionsProcessor::new(&mut client_mgr, &mut transaction_mgr, loader);

        let mut action = TransactionDetails {
            transaction_type: TransactionType::Withdrawal,
            client: 2,
            tx: 1,
            amount: None,
        };

        assert!(
            !mgr.apply_withdrawal(action.clone()),
            "Should be failed as amount is not provided!"
        );
        assert!(mgr.client_state_mgr.get_states().is_empty());
        assert!(!mgr.transaction_mgr.transaction_exist(1));

        action.amount = Some(13.);
        assert!(
            !mgr.apply_withdrawal(action.clone()),
            "Total can't be negative: 0-13."
        );

        let mut deposit = action.clone();
        deposit.transaction_type = TransactionType::Deposit;
        deposit.amount = Some(9.5);
        assert!(mgr.apply_deposit(deposit.clone())); // Amount == 9.5

        assert!(
            !mgr.apply_withdrawal(action.clone()),
            "Tx amount more than available!"
        );
        action.amount = Some(7.);
        assert!(
            !mgr.apply_withdrawal(action.clone()),
            "Tx id is not unique!"
        );
        action.tx = 4;
        assert!(mgr.apply_withdrawal(action.clone()));

        let clients = mgr.client_state_mgr.get_states();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].client, 2);
        assert!(approx_eq!(f32, clients[0].available, 2.5, ulps = 4));
        assert!(approx_eq!(f32, clients[0].held, 0., ulps = 4));
        assert!(approx_eq!(f32, clients[0].total, 2.5, ulps = 4));
        assert!(!clients[0].locked);
        assert!(!mgr.apply_withdrawal(deposit), "Type mismatch");

        action.amount = Some(1.);
        assert!(
            !mgr.apply_withdrawal(action.clone()),
            "Tx id is not unique."
        );

        action.tx = 3;
        assert!(mgr.apply_withdrawal(action.clone()));
        let clients = mgr.client_state_mgr.get_states();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].client, 2);
        assert!(approx_eq!(f32, clients[0].available, 1.5, ulps = 4));
        assert!(approx_eq!(f32, clients[0].held, 0., ulps = 4));
        assert!(approx_eq!(f32, clients[0].total, 1.5, ulps = 4));
        assert!(!clients[0].locked);

        action.amount = Some(3.);
        action.tx = 4;
        assert!(
            !mgr.apply_withdrawal(action.clone()),
            "Tx amount more than available!"
        );
    }
}
