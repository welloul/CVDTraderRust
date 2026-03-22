use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use crate::core::state::GlobalState;
use crate::execution::gateway::ExecutionGateway;

pub struct OrderTTLTracker {
    pub gateway: Option<Arc<Mutex<ExecutionGateway>>>,
    // state: Arc<Mutex<GlobalState>>,
}

impl OrderTTLTracker {
    pub fn new(_state: Arc<Mutex<GlobalState>>, _gateway: Option<Arc<Mutex<Self>>>) -> Self {
        Self {
            gateway: None,
        }
    }

    pub async fn start(&mut self) {
//         println!("[INFO] "OrderTTLTracker start not implemented");
    }
}