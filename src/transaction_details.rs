use serde::Serialize;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

/// Supported transaction types
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Debug)]
#[serde(into = "&str", from = "&str")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
    Unknown,
}

/// Base transaction details
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransactionDetails {
    /// The transaction type
    pub transaction_type: TransactionType,
    /// Unique Id of the client
    #[serde(deserialize_with = "u16_with_whitespace")]
    pub client: u16,
    /// The transaction Id
    #[serde(deserialize_with = "u32_with_whitespace")]
    pub tx: u32,
    /// Amount of transaction - only for deposit and withdrawal
    #[serde(deserialize_with = "f32_with_whitespace")]
    pub amount: Option<f32>,
}

/// String to transaction type conversion
impl From<&str> for TransactionType {
    fn from(type_str: &str) -> Self {
        match type_str.trim() {
            "deposit" => TransactionType::Deposit,
            "withdrawal" => TransactionType::Withdrawal,
            "dispute" => TransactionType::Dispute,
            "resolve" => TransactionType::Resolve,
            "chargeback" => TransactionType::Chargeback,
            _ => TransactionType::Unknown,
        }
    }
}

/// Transaction type to string conversion
impl From<TransactionType> for &str {
    fn from(transaction_type: TransactionType) -> Self {
        match transaction_type {
            TransactionType::Deposit => "deposit",
            TransactionType::Withdrawal => "withdrawal",
            TransactionType::Dispute => "dispute",
            TransactionType::Resolve => "resolve",
            TransactionType::Chargeback => "chargeback",
            TransactionType::Unknown => "unknown",
        }
    }
}

/// To handle cases when i16 contains whitespaces in csv
fn u16_with_whitespace<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    u16::from_str(buf.trim()).map_err(serde::de::Error::custom)
}

/// To handle cases when the digit contains whitespaces
fn u32_with_whitespace<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    u32::from_str(buf.trim()).map_err(serde::de::Error::custom)
}

/// To handle cases when f32 digit contains whitespaces
fn f32_with_whitespace<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf: Option<String> = Option::deserialize(deserializer)?;
    match buf {
        Some(d) => Ok(Some(
            f32::from_str(d.trim()).map_err(serde::de::Error::custom)?,
        )),
        None => Ok(None),
    }
}
