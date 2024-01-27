use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SubscriptionID {
    WORKSPACE,
    WINDOW,
    SINKVOLUME,
    SOURCEVOLUME,
}

#[derive(Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub pid: u32,
    pub name: String,
}
