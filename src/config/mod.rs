use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{balance::AssetId, engine::models::market::PairId};

pub mod repositories;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub markets: Vec<MarketConfig>
}

#[derive(Debug, Deserialize)]
pub struct MarketConfig {
    pub pair_id: PairId,
    pub base_asset_id: AssetId,
    pub quote_asset_id: AssetId,
    pub is_market_trade_enabled: bool,
    pub min_allowed_quantity: Decimal,
}