use crate::ClientState;
use std::collections::HashMap;

/// Interface to manage clients states
pub struct ClientsStatesMgr {
    clients_states: HashMap<u16, ClientState>,
}

/// Note: Now we allow double actions on locked account - to skip uncomment get_client_details
impl ClientsStatesMgr {
    /// Create state manager
    pub fn new() -> Self {
        Self {
            clients_states: Default::default(),
        }
    }

    /// Get current states of all clients
    pub fn get_states(&self) -> Vec<ClientState> {
        self.clients_states.values().cloned().collect()
    }

    /// Apply deposit - i.e. increase available funds
    /// returns state of the operation - now it always true/success
    pub fn apply_deposit(&mut self, client_id: u16, amount: f32) -> bool {
        let data = self
            .clients_states
            .entry(client_id)
            .or_insert_with(|| ClientState {
                client: client_id,
                available: 0.0,
                held: 0.0,
                total: 0.0,
                locked: false,
            });

        data.available += amount;
        data.total += amount;

        true
    }

    /// Apply withdrawal on clients account - decrease funds
    /// returns state of the operation - false if can't apply withdrawal
    pub fn apply_withdrawal(&mut self, client_id: u16, amount: f32) -> bool {
        let data = self
            .get_client_details(client_id)
            // available amount shouldn't be less!
            .filter(|d| d.available >= amount);

        if data.is_none() {
            return false;
        }

        let data = data.unwrap();

        data.available -= amount;
        data.total -= amount;

        true
    }

    /// Apply dispute on client state
    /// Returns state - false if client is not present of available less than the amount
    pub fn apply_dispute(&mut self, client_id: u16, amount: f32) -> bool {
        let data = self
            .get_client_details(client_id)
            .filter(|d| d.available >= amount);
        if data.is_none() {
            return false;
        }

        let data = data.unwrap();

        data.available -= amount;
        data.held += amount;

        true
    }

    /// Apply resolve on client state
    /// Returns state - false if client is not present of held less than the amount
    pub fn apply_resolve(&mut self, client_id: u16, amount: f32) -> bool {
        let data = self
            .get_client_details(client_id)
            .filter(|d| d.held >= amount);
        if data.is_none() {
            return false;
        }

        let data = data.unwrap();

        data.available += amount;
        data.held -= amount;

        true
    }

    /// Apply chargeback on client's state and mark the account as locked
    /// State of the operation - failed if client is not present or held less than the amount
    pub fn apply_chargeback(&mut self, client_id: u16, amount: f32) -> bool {
        let data = self
            .get_client_details(client_id)
            .filter(|d| d.held >= amount);
        if data.is_none() {
            return false;
        }

        let data = data.unwrap();

        data.total -= amount;
        data.held -= amount;
        data.locked = true;

        true
    }

    fn get_client_details(&mut self, client_id: u16) -> Option<&mut ClientState> {
        self.clients_states.get_mut(&client_id)
        // Enable if we need to eliminate actions on locked client account!
        //.filter(|d| !d.locked)
    }
}

#[cfg(test)]
mod test {
    use crate::ClientsStatesMgr;
    use float_cmp::approx_eq;

    #[test]
    pub fn test_deposits() {
        let mut mgr = ClientsStatesMgr::new();

        assert!(mgr.apply_deposit(2, 13.));
        let c = mgr.clients_states.get(&2);
        assert!(c.is_some(), "New client should be added!");
        let c = c.unwrap();
        assert_eq!(c.client, 2);
        assert!(!c.locked, "New added client shouldn't be locked");
        assert!(approx_eq!(f32, c.total, 13., ulps = 4));
        assert!(approx_eq!(f32, c.available, 13., ulps = 4));
        assert!(
            approx_eq!(f32, c.held, 0., ulps = 4),
            "In case of deposit held shouldn't be updated!"
        );

        assert!(mgr.apply_deposit(2, 15.));
        let c = mgr.clients_states.get(&2).unwrap();
        assert_eq!(mgr.clients_states.len(), 1, "Old client should be updated!");
        assert_eq!(c.client, 2);
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 28., ulps = 4));
        assert!(approx_eq!(f32, c.available, 28., ulps = 4));
        assert!(
            approx_eq!(f32, c.held, 0., ulps = 4),
            "In case of deposit held shouldn't be updated!"
        );

        assert!(mgr.apply_deposit(3, 17.));
        assert_eq!(mgr.clients_states.len(), 2, "New client should be added!");
        let c3 = mgr.clients_states.get(&3).unwrap();
        assert_eq!(c3.client, 3);
        assert!(!c3.locked);
        assert!(approx_eq!(f32, c3.total, 17., ulps = 4));
        assert!(approx_eq!(f32, c3.available, 17., ulps = 4));
        assert!(
            approx_eq!(f32, c3.held, 0., ulps = 4),
            "In case of deposit held shouldn't be updated!"
        );

        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert_eq!(c.client, 2);
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 28., ulps = 4));
        assert!(approx_eq!(f32, c.available, 28., ulps = 4));
        assert!(approx_eq!(f32, c.held, 0., ulps = 4));

        c.held = 11.;
        c.total += 11.;
        assert!(mgr.apply_deposit(2, 17.));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert_eq!(c.client, 2);
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 56., ulps = 4));
        assert!(approx_eq!(f32, c.available, 45., ulps = 4));
        assert!(
            approx_eq!(f32, c.held, 11., ulps = 4),
            "Held value shouldn't be changed!"
        );

        assert_eq!(
            mgr.get_states().len(),
            2,
            "Should return both client details!"
        );
    }

    #[test]
    pub fn test_withdraw() {
        let mut mgr = ClientsStatesMgr::new();
        assert!(
            !mgr.apply_withdrawal(2, 1.),
            "Should be failed as no client available!"
        );
        assert!(mgr.clients_states.is_empty(), "Nth. should be added!");

        assert!(mgr.apply_deposit(2, 11.));
        assert!(
            !mgr.apply_withdrawal(2, 12.),
            "Should be failed as available amount is more!"
        );
        assert!(
            mgr.apply_withdrawal(2, 9.),
            "Should be fine as available fund is higher "
        );

        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert_eq!(c.client, 2);
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 2., ulps = 4));
        assert!(approx_eq!(f32, c.available, 2., ulps = 4));
        assert!(
            approx_eq!(f32, c.held, 0., ulps = 4),
            "Held value shouldn't be changed!"
        );
        assert!(!mgr.apply_withdrawal(3, 2.), "No client data!");

        let c = mgr.clients_states.get_mut(&2).unwrap();
        c.held = 3.;
        c.total += 3.;

        assert!(mgr.apply_withdrawal(2, 1.5));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert_eq!(c.client, 2);
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 3.5, ulps = 4));
        assert!(approx_eq!(f32, c.available, 0.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 3., ulps = 4));

        assert!(mgr.apply_withdrawal(2, 0.5), "Available == 0.5 -> ok");
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 3.0, ulps = 4));
        assert!(approx_eq!(f32, c.available, 0.0, ulps = 4));
        assert!(approx_eq!(f32, c.held, 3., ulps = 4));
    }

    #[test]
    pub fn test_dispute() {
        let mut mgr = ClientsStatesMgr::new();
        assert!(
            !mgr.apply_dispute(2, 1.),
            "Should be failed as no client available!"
        );

        mgr.apply_deposit(2, 11.5);

        assert!(mgr.apply_dispute(2, 2.));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 11.5, ulps = 4));
        assert!(approx_eq!(f32, c.available, 9.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 2., ulps = 4));

        assert!(mgr.apply_dispute(2, 9.));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 11.5, ulps = 4));
        assert!(approx_eq!(f32, c.available, 0.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 11., ulps = 4));

        assert!(!mgr.apply_dispute(3, 1.), "There is no client 3!");

        assert!(!mgr.apply_dispute(2, 1.), "No 1.0 available!");
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 11.5, ulps = 4));
        assert!(approx_eq!(f32, c.available, 0.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 11., ulps = 4));

        assert!(mgr.apply_dispute(2, 0.5));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 11.5, ulps = 4));
        assert!(approx_eq!(f32, c.available, 0., ulps = 4));
        assert!(approx_eq!(f32, c.held, 11.5, ulps = 4));

        assert!(!mgr.apply_dispute(2, 0.1));
    }

    #[test]
    pub fn test_resolve() {
        let mut mgr = ClientsStatesMgr::new();
        assert!(
            !mgr.apply_resolve(2, 1.),
            "Should be failed as no client available!"
        );

        mgr.apply_deposit(2, 2.5);
        assert!(
            !mgr.apply_resolve(2, 1.),
            "Should be failed as held is 0 -> <2.5!"
        );

        mgr.clients_states.get_mut(&2).unwrap().held = 3.5;
        mgr.clients_states.get_mut(&2).unwrap().total = 6.;
        assert!(mgr.apply_resolve(2, 1.));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(!c.locked);
        assert!(approx_eq!(f32, c.total, 6., ulps = 4));
        assert!(approx_eq!(f32, c.available, 3.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 2.5, ulps = 4));

        assert!(mgr.apply_resolve(2, 2.5));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(approx_eq!(f32, c.total, 6., ulps = 4));
        assert!(approx_eq!(f32, c.available, 6., ulps = 4));
        assert!(approx_eq!(f32, c.held, 0., ulps = 4));

        assert!(!mgr.apply_resolve(2, 0.5), "Held == 0");
    }

    #[test]
    pub fn test_chargeback() {
        let mut mgr = ClientsStatesMgr::new();
        assert!(
            !mgr.apply_chargeback(2, 1.),
            "Should be failed as no client available!"
        );

        mgr.apply_deposit(2, 2.5);
        assert!(
            !mgr.apply_chargeback(2, 1.),
            "Should be failed as held == 0!"
        );
        mgr.clients_states.get_mut(&2).unwrap().held = 3.5;
        mgr.clients_states.get_mut(&2).unwrap().total = 6.;

        assert!(mgr.apply_chargeback(2, 1.));

        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(c.locked, "Should be marked as locked!");
        assert!(approx_eq!(f32, c.total, 5., ulps = 4));
        assert!(approx_eq!(f32, c.available, 2.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 2.5, ulps = 4));

        assert!(mgr.apply_chargeback(2, 2.5));
        let c = mgr.clients_states.get_mut(&2).unwrap();
        assert!(c.locked, "Should remain as locked after chargeback!");
        assert!(approx_eq!(f32, c.total, 2.5, ulps = 4));
        assert!(approx_eq!(f32, c.available, 2.5, ulps = 4));
        assert!(approx_eq!(f32, c.held, 0., ulps = 4));
        assert!(!mgr.apply_chargeback(2, 2.5));
    }
}
