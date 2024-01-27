use super::data::HyprvisorData;
use crate::protocols::subscription::SubscriptionID;
use std::collections::HashMap;
use tokio::net::UnixStream;

pub struct HyprvisorState {
    pub data: HyprvisorData,
    pub subscribers: HashMap<SubscriptionID, HashMap<u32, UnixStream>>,
}

impl HyprvisorState {
    pub fn new() -> Self {
        HyprvisorState {
            data: HyprvisorData::new(),
            subscribers: HashMap::new(),
        }
    }
}
