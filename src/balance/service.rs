use std::sync::Arc;

use rust_decimal::Decimal;

use crate::common::{
    errors::{AppError, AppResult},
    time::{Time, Timestamp},
};

use super::{AssetId, BalanceSourceExector, BalanceStatus, BalanceType, UserId};

pub type BalanceSource = Box<dyn BalanceSourceExector>;

#[derive(Clone, Copy)]
pub enum BusinessType {
    Withdraw,
    Deposit,
    Trade,
}

pub type BusinessId = u64;

pub struct ChangeBalanceOutput {
    pub user_id: UserId,
    pub asset_id: AssetId,
    pub business_type: BusinessType,
    pub business_id: BusinessId,
    pub balance_type: BalanceType,
    pub amount: Decimal,
    pub total_balance: Decimal,
    pub available_balance: Decimal,
    pub frozen_balance: Decimal,
    pub created_at: Timestamp,
}

pub struct BalanceService {
    source: Arc<BalanceSource>,
}

impl BalanceService {
    pub fn new(source: Arc<BalanceSource>) -> Self {
        Self { source }
    }

    pub fn is_available_balance_enough(
        &self,
        user_id: UserId,
        asset_id: AssetId,
        amount: Decimal,
    ) -> bool {
        let balance = self.source.get(user_id, BalanceType::Available, asset_id);

        if balance.lt(&amount) {
            return false;
        }

        true
    }

    pub fn get_balance_status(&self, user_id: UserId, asset_id: AssetId) -> BalanceStatus {
        self.source.get_status(user_id, asset_id)
    }

    pub fn change_balance(
        &self,
        user_id: UserId,
        asset_id: AssetId,
        business_type: BusinessType,
        business_id: BusinessId,
        balance_type: BalanceType,
        amount: Decimal,
    ) -> AppResult<ChangeBalanceOutput> {
        let abs_amount = amount.abs();

        if amount.is_sign_positive() {
            self.source
                .increase(user_id, balance_type, asset_id, abs_amount);
        } else {
            let total_balance = self.source.get(user_id, balance_type, asset_id);

            if total_balance.lt(&abs_amount) {
                return Err(AppError::UserBalanceExceeds);
            }

            self.source
                .decrease(user_id, balance_type, asset_id, abs_amount);
        }

        let balance_status = self.source.get_status(user_id, asset_id);

        Ok(ChangeBalanceOutput {
            user_id: user_id,
            asset_id: asset_id,
            business_type: business_type,
            business_id: business_id,
            balance_type: balance_type,
            amount: amount,
            total_balance: balance_status.total,
            available_balance: balance_status.available,
            frozen_balance: balance_status.frozen,
            created_at: Time::get_current_timestamp(),
        })
    }
}
