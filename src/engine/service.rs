use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    vec,
};

use rust_decimal::{prelude::Zero, Decimal};

use crate::{
    balance::{service::BalanceService, UserId},
    common::errors::{AppError, AppResult},
    config::{self, service::ConfigService},
};

use super::models::{
    market::{Market, PairId},
    order::{OrderPrice, OrderSide},
    orderbook::OrderbookDepth,
};

pub type Markets = HashMap<PairId, Market>;

pub struct EngineService {
    markets: RwLock<Markets>,
    balance_service: Arc<BalanceService>,
}

impl EngineService {
    pub fn new(balance_service: Arc<BalanceService>) -> Self {
        Self {
            markets: RwLock::new(HashMap::new()),
            balance_service,
        }
    }

    pub fn insert_markets_from_config(&mut self, config_service: &ConfigService) {
        let mut write_guard = self.markets.try_write().unwrap();

        for market_config in &config_service.markets {
            let market = Market::new(
                market_config.base_asset_id,
                market_config.quote_asset_id,
                market_config.is_market_trade_enabled,
                market_config.min_allowed_quantity,
                self.balance_service.clone(),
            );

            write_guard.insert(market_config.pair_id, market);
        }
    }

    pub fn get_market_orderbook(&self, pair_id: PairId) -> (OrderbookDepth, OrderbookDepth) {
        if let Some(market) = self.markets.try_read().unwrap().get(&pair_id) {
            return market.get_orderbook_depth();
        }

        (
            vec![[Decimal::zero(), Decimal::zero()]],
            vec![[Decimal::zero(), Decimal::zero()]],
        )
    }

    pub fn place_order(
        &self,
        pair_id: PairId,
        user_id: UserId,
        limit_price: Option<OrderPrice>,
        quantity: Decimal,
        side: OrderSide,
    ) -> AppResult<()> {
        if let Some(market) = self.markets.try_write().unwrap().get_mut(&pair_id) {
            market.process_new_order(user_id, limit_price, quantity, side)?;
        }

        Err(AppError::MarketNotFound)
    }
}
