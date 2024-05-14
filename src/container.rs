use std::sync::Arc;

use crate::{balance::{repositories::memory::MemoryBalanceManager, service::BalanceService, BalanceSourceExector}, engine::service::EngineService};

pub struct Container {
    pub balance_service: Arc<BalanceService>,
    pub engine_service: Arc<EngineService>
}

impl Container {
    pub fn new() -> Self {
        let balance_source: Arc<Box<dyn BalanceSourceExector>> = Arc::new(Box::new(MemoryBalanceManager::new()));
        let balance_service = Arc::new(BalanceService::new(balance_source.clone()));

        let engine_service = Arc::new(EngineService::new());

        Self {
            balance_service,
            engine_service
        }
    }
}