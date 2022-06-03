use serde::Deserialize;
use serde::Serialize;

/// Current client state
#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub struct ClientState {
    /// Client id.
    pub client: u16,
    /// The total funds that are available for trading, staking, withdrawal, etc.
    pub available: f32,
    /// The total funds that are held for dispute.
    pub held: f32,
    /// The total funds that are available or held.
    pub total: f32,
    /// Whether the account is locked.
    pub locked: bool,
}
