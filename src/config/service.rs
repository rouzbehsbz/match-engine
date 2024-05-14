use rust_decimal::Decimal;

use crate::{balance::AssetId, engine::models::market::PairId};

pub struct MarketConfig {
    pub pair_id: PairId,
    pub base_asset_id: AssetId,
    pub quote_asset_id: AssetId,
    pub is_market_trade_enabled: bool,
    pub min_allowed_quantity: Decimal
}

pub struct ConfigService {
    pub markets: Vec<MarketConfig>
}

impl ConfigService {
    pub fn new() -> Self {
        Self {
            markets: vec![]
        }
    }
}