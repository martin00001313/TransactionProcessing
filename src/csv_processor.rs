use crate::transaction_details::TransactionDetails;
use anyhow::anyhow;
use serde::Serialize;

/// Base trait for transaction details fetchers
/// Data can be fetched  from file, cloud, web source, etc.
pub trait TransactionLoader {
    fn next_transaction(&mut self) -> Option<TransactionDetails>;
}

/// Base fetcher to upload data from the file
pub struct TransactionIOLoader {
    transaction_records: Vec<TransactionDetails>,
    curr_idx: usize,
}

impl TransactionIOLoader {
    /// Create new transaction loader based on the provided transaction file
    pub fn new(transaction_path: &str) -> Result<Self, anyhow::Error> {
        let mut reader = csv::Reader::from_path(transaction_path).map_err(|e| anyhow!(e))?;

        let mut transactions = Vec::new();
        for record in reader.deserialize() {
            let record: TransactionDetails = record?;
            transactions.push(record);
        }

        Ok(Self {
            transaction_records: transactions,
            curr_idx: 0,
        })
    }
}

impl TransactionLoader for TransactionIOLoader {
    /// Get next transaction details
    fn next_transaction(&mut self) -> Option<TransactionDetails> {
        match self.transaction_records.get(self.curr_idx) {
            Some(data) => {
                self.curr_idx += 1;
                Some(data.clone())
            }
            None => None,
        }
    }
}

/// Generate csv content from provided data
pub fn generate_csv<W>(clients_details: &Vec<W>) -> Result<String, anyhow::Error>
where
    W: Serialize,
{
    let mut writer = csv::Writer::from_writer(Vec::new());

    for d in clients_details {
        writer.serialize(d)?;
    }

    Ok(String::from_utf8(writer.into_inner()?)?)
}
