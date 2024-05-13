use std::{collections::HashMap, sync::RwLock};

use rust_decimal::{prelude::Zero, Decimal};

use crate::balance::{AssetId, BalanceSourceExector, BalanceStatus, BalanceType, UserId};

#[derive(Hash, PartialEq, Eq)]
pub struct BalancesKey {
    user_id: UserId,
    asset_id: AssetId,
    type_: BalanceType
}

pub type Balances = HashMap<BalancesKey, Decimal>;

pub struct MemoryBalanceManager {
    balances: RwLock<Balances>
}

impl MemoryBalanceManager {
    pub fn new() -> Self {
        Self {
            balances: RwLock::new(HashMap::new())
        }
    }

    pub fn get_by_key(&self, key: &BalancesKey) -> Decimal {
        *self.balances.try_read().unwrap().get(key).unwrap_or(&Decimal::zero())
    }

    pub fn set_by_key(&self, key: BalancesKey, amount: Decimal) {
        self.balances.try_write().unwrap().insert(key, amount);
    }

    pub fn set(&self,  user_id: UserId, type_: BalanceType, asset_id: AssetId, amount: Decimal) {
        self.set_by_key(BalancesKey {
            user_id,
            asset_id,
            type_
        }, amount);
    }
}

impl BalanceSourceExector for MemoryBalanceManager {
    fn get(&self, user_id: UserId, type_: BalanceType, asset_id: AssetId) -> Decimal {
        self.get_by_key(&BalancesKey {
            user_id,
            asset_id,
            type_
        })
    }

    fn increase(&self, user_id: UserId, type_: BalanceType, asset_id: AssetId, amount: Decimal) {
        let old_amount = self.get(user_id, type_, asset_id);
        let new_amount = old_amount + amount;

        self.set(user_id, type_, asset_id, new_amount)
    }

    fn decrease(&self, user_id: UserId, type_: BalanceType, asset_id: AssetId, amount: Decimal) {
        let old_amount = self.get(user_id, type_, asset_id);
        let new_amount = old_amount - amount;

        self.set(user_id, type_, asset_id, new_amount)
    }

    fn get_total(&self, user_id: UserId, asset_id: AssetId) -> Decimal {
        self.get(user_id, BalanceType::Available, asset_id) + self.get(user_id, BalanceType::Frozen, asset_id)
    }

    fn get_status(&self, user_id: UserId, asset_id: AssetId) -> BalanceStatus {
        let available = self.get(user_id, BalanceType::Available, asset_id);
        let frozen = self.get(user_id, BalanceType::Frozen, asset_id);
        let total = self.get_total(user_id, asset_id);

        BalanceStatus {
            available,
            frozen,
            total
        }
    }
}