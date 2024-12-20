use rust_decimal::Decimal;

pub mod repositories;
pub mod service;

pub type UserId = u32;
pub type AssetId = u32;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BalanceType {
    Available,
    Frozen,
}

#[derive(Debug)]
pub struct BalanceStatus {
    pub total: Decimal,
    pub available: Decimal,
    pub frozen: Decimal,
}

pub trait BalanceSourceExector: Send + Sync {
    fn get(&self, user_id: UserId, type_: BalanceType, asset_id: AssetId) -> Decimal;
    fn increase(&self, user_id: UserId, type_: BalanceType, asset_id: AssetId, amount: Decimal);
    fn decrease(&self, user_id: UserId, type_: BalanceType, asset_id: AssetId, amount: Decimal);
    fn get_total(&self, user_id: UserId, asset_id: AssetId) -> Decimal;
    fn get_status(&self, user_id: UserId, asset_id: AssetId) -> BalanceStatus;
}
