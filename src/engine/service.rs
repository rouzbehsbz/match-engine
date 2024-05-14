use std::collections::HashMap;

use super::market::{Market, PairId};

pub type Markets = HashMap<PairId, Market>;

pub struct EngineService {
    markets: Markets
}

impl EngineService {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new()
        }
    }
}