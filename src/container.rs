use std::sync::Arc;

use crate::{
    balance::{
        repositories::memory::MemoryBalanceManager, service::BalanceService, BalanceSourceExector,
    },
    common::sequencer::Sequencer,
    config::Config,
    engine::service::EngineService,
};

pub struct Container {
    pub balance_service: Arc<BalanceService>,
    pub engine_service: Arc<EngineService>,
}

impl Container {
    pub fn new(config: &Config) -> Self {
        let balance_source: Arc<Box<dyn BalanceSourceExector>> =
            Arc::new(Box::new(MemoryBalanceManager::new()));
        let balance_service = Arc::new(BalanceService::new(balance_source.clone()));

        let mut engine_service = EngineService::new(balance_service.clone());

        engine_service.insert_markets_from_config(config);

        Self {
            balance_service,
            engine_service: Arc::new(engine_service),
        }
    }
}
