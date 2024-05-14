use std::{collections::HashMap, sync::RwLock, vec};

use rust_decimal::{prelude::Zero, Decimal};

use crate::{balance::UserId, common::errors::{AppError, AppResult}};

use super::models::{market::{Market, PairId}, order::{OrderPrice, OrderSide}, orderbook::OrderbookDepth};

pub type Markets = HashMap<PairId, Market>;

pub struct EngineService {
    markets: RwLock<Markets>,
}

impl EngineService {
    pub fn new() -> Self {
        Self {
            markets: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_market_orderbook(&self, pair_id: PairId) -> (OrderbookDepth, OrderbookDepth) {
        if let Some(market) = self.markets.try_read().unwrap().get(&pair_id) {
            return market.get_orderbook_depth()
        }

        (vec![[Decimal::zero(), Decimal::zero()]], vec![[Decimal::zero(), Decimal::zero()]])
    }

    pub fn place_order(&self, pair_id: PairId, user_id: UserId, limit_price: Option<OrderPrice>, quantity: Decimal, side: OrderSide) -> AppResult<()> {
        if let Some(market) = self.markets.try_write().unwrap().get_mut(&pair_id) {
            market.process_new_order(user_id, limit_price, quantity, side)?;
        }

        Err(AppError::MarketNotFound)
    }
}
