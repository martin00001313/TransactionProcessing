use crate::client_state::ClientState;
use crate::client_state_mgr::ClientsStatesMgr;
use crate::csv_processor::{generate_csv, TransactionIOLoader};
use crate::transaction_details::{TransactionDetails, TransactionType};
use crate::transaction_mgr::TransactionMgr;
use crate::transaction_processor::TransactionsProcessor;

mod client_state;
mod client_state_mgr;
mod csv_processor;
mod transaction_details;
mod transaction_mgr;
mod transaction_processor;

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        return;
    }

    let transactions_path = match args.nth(1) {
        Some(path) => path,
        None => return,
    };

    match run_flow(&transactions_path) {
        Ok(csv_data) => println!("{}", csv_data.as_str()),
        Err(e) => eprintln!("{:?}", e),
    }
}

/// Run the workflow
fn run_flow(path: &str) -> Result<String, anyhow::Error> {
    let mut client_state_mgr = ClientsStatesMgr::new();
    let mut transaction_mgr = TransactionMgr::new();
    let mut transaction_actions_processor = TransactionsProcessor::new(
        &mut client_state_mgr,
        &mut transaction_mgr,
        TransactionIOLoader::new(path)?,
    );

    transaction_actions_processor.apply_transaction_actions();

    generate_csv(&client_state_mgr.get_states())
}

#[cfg(test)]
mod test {
    use crate::{run_flow, ClientState};
    use float_cmp::approx_eq;
    use std::collections::HashMap;

    #[test]
    pub fn test_flow() {
        let path = "./src/test_utils/transactions.csv";
        let result = run_flow(path);

        assert!(result.is_ok());

        let result = result.unwrap();
        let mut rdr = csv::Reader::from_reader(result.as_bytes());
        let mut data = Vec::new();
        for r in rdr.deserialize() {
            let r: ClientState = r.unwrap();
            data.push(r);
        }

        let id_to_data: HashMap<u16, ClientState> =
            data.into_iter().map(|d| (d.client, d)).collect();
        assert_eq!(id_to_data.len(), 3);

        let c3 = id_to_data.get(&3).unwrap();
        assert!(c3.locked, "Should be locked due to chargeback!");
        assert!(approx_eq!(f32, c3.total, 11.5, ulps = 4));
        assert!(approx_eq!(f32, c3.available, 11.5, ulps = 4));
        assert!(approx_eq!(f32, c3.held, 0., ulps = 4));

        let c5 = id_to_data.get(&5).unwrap();
        assert!(
            !c5.locked,
            "Should not be locked due to incorrect chargeback!"
        );
        assert!(approx_eq!(f32, c5.total, 32.3343, ulps = 4));
        assert!(approx_eq!(f32, c5.held, 0., ulps = 4));
        assert!(approx_eq!(f32, c5.available, 32.3343, ulps = 4));

        let c1 = id_to_data.get(&1).unwrap();
        assert!(
            !c1.locked,
            "Should not be locked as there is no chargeback!"
        );
        assert!(approx_eq!(f32, c1.total, 28., ulps = 4));
        assert!(approx_eq!(f32, c1.held, 0., ulps = 4));
        assert!(approx_eq!(f32, c1.available, 28., ulps = 4));
    }
}
