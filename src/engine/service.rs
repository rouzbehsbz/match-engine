use std::{collections::HashMap, vec};

use rust_decimal::{prelude::Zero, Decimal};

use super::{market::{Market, PairId}, order::OrderPrice, orderbook::OrderbookDepth};

pub type Markets = HashMap<PairId, Market>;

pub struct EngineService {
    markets: Markets,
}

impl EngineService {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new(),
        }
    }

    pub fn get_market_orderbook(&self, pair_id: PairId) -> (OrderbookDepth, OrderbookDepth) {
        if let Some(market) = self.markets.get(&pair_id) {
            return market.get_orderbook_depth()
        }

        (vec![[Decimal::zero(), Decimal::zero()]], vec![[Decimal::zero(), Decimal::zero()]])
    }
}
