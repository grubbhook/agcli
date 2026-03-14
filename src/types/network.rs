//! Network identifiers and connection presets.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Subnet UID (u16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NetUid(pub u16);

impl NetUid {
    /// Root network (netuid 0).
    pub const ROOT: Self = Self(0);

    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for NetUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for NetUid {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

/// Well-known Bittensor networks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Network {
    #[default]
    Finney,
    Test,
    Local,
    Custom(String),
}

impl Network {
    /// WebSocket endpoint URL.
    pub fn ws_url(&self) -> &str {
        match self {
            Network::Finney => "wss://entrypoint-finney.opentensor.ai:443",
            Network::Test => "wss://test.finney.opentensor.ai:443",
            Network::Local => "ws://127.0.0.1:9944",
            Network::Custom(url) => url,
        }
    }

    /// SS58 address prefix for this network.
    pub fn ss58_prefix(&self) -> u16 {
        42 // Bittensor uses prefix 42
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Network::Finney => write!(f, "finney"),
            Network::Test => write!(f, "test"),
            Network::Local => write!(f, "local"),
            Network::Custom(url) => write!(f, "custom({})", url),
        }
    }
}
