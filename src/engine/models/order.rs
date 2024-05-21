use rust_decimal::{prelude::Zero, Decimal};

use crate::{
    balance::{AssetId, UserId},
    common::errors::{AppError, AppResult},
};

pub type OrderId = u64;
pub type OrderPrice = Decimal;
pub type OrderQuantity = Decimal;
pub type OrderAmount = Decimal;

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Limit { price: OrderPrice },
    Market,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderSide {
    Ask,
    Bid,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Cancelled,
    Closed,
    Filled,
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    id: OrderId,
    user_id: UserId,
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    type_: OrderType,
    side: OrderSide,
    quantity: Decimal,
    filled_quantity: Decimal,
    frozen_amount: Decimal,
    status: OrderStatus,
}

impl Order {
    pub fn new_limit(
        id: OrderId,
        user_id: UserId,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        side: OrderSide,
        limit_price: OrderPrice,
        quantity: OrderQuantity,
    ) -> Self {
        Self {
            id,
            user_id,
            base_asset_id,
            quote_asset_id,
            type_: OrderType::Limit { price: limit_price },
            side,
            quantity,
            filled_quantity: Decimal::zero(),
            frozen_amount: Decimal::zero(),
            status: OrderStatus::Open,
        }
    }

    pub fn new_market(
        id: OrderId,
        user_id: UserId,
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        side: OrderSide,
        quantity: OrderQuantity,
    ) -> Self {
        Self {
            id,
            user_id,
            base_asset_id,
            quote_asset_id,
            type_: OrderType::Market,
            side,
            quantity,
            filled_quantity: Decimal::zero(),
            frozen_amount: Decimal::zero(),
            status: OrderStatus::Open,
        }
    }

    pub fn get_user_id(&self) -> UserId {
        self.user_id
    }

    pub fn get_base_asset_id(&self) -> AssetId {
        self.base_asset_id
    }

    pub fn get_quote_asset_id(&self) -> AssetId {
        self.quote_asset_id
    }

    pub fn get_asset_id(&self) -> AssetId {
        match self.get_side() {
            OrderSide::Ask => self.get_base_asset_id(),
            OrderSide::Bid => self.get_quote_asset_id(),
        }
    }

    pub fn get_id(&self) -> OrderId {
        self.id
    }

    pub fn get_limit_price(&self) -> Option<OrderPrice> {
        match self.type_ {
            OrderType::Limit { price } => Some(price),
            OrderType::Market => None,
        }
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn get_remaining_quantity(&self) -> OrderQuantity {
        self.quantity - self.filled_quantity
    }

    pub fn get_quantity(&self) -> OrderQuantity {
        self.quantity
    }

    pub fn get_amount(&self) -> AppResult<OrderAmount> {
        let limit_price = self
            .get_limit_price()
            .ok_or(AppError::InvalidMarketOrderAmount)?;

        Ok(self.get_quantity() * limit_price)
    }

    pub fn get_traded_quantity(&self, matched_order: &Order) -> OrderQuantity {
        self.get_remaining_quantity()
            .min(matched_order.get_remaining_quantity())
    }

    pub fn fill(&mut self, quantity: OrderQuantity) -> AppResult<()> {
        if quantity > self.get_remaining_quantity() {
            return Err(AppError::OrderOverFilled);
        }

        self.filled_quantity += quantity;
        self.status = if self.filled_quantity == self.quantity {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        match self.status {
            OrderStatus::Cancelled | OrderStatus::Closed | OrderStatus::Filled => true,
            _ => false,
        }
    }

    pub fn is_bookable(&self) -> bool {
        match self.type_ {
            OrderType::Limit { .. } => true,
            OrderType::Market { .. } => false,
        }
    }

    pub fn get_frozen_amount(&self) -> Decimal {
        self.frozen_amount
    }

    pub fn decrease_frozen_amount(&mut self, traded_quantity: OrderQuantity) -> AppResult<()> {
        match self.get_side() {
            OrderSide::Ask => self.frozen_amount -= traded_quantity,
            OrderSide::Bid => {
                let limit_price = self
                    .get_limit_price()
                    .ok_or(AppError::OrderInavlidFrozenAmount)?;

                self.frozen_amount -= traded_quantity * limit_price
            }
        }

        Ok(())
    }

    pub fn set_frozen_amount(&mut self) -> AppResult<()> {
        match self.get_side() {
            OrderSide::Ask => self.frozen_amount = self.get_remaining_quantity(),
            OrderSide::Bid => {
                let limit_price = self
                    .get_limit_price()
                    .ok_or(AppError::OrderInavlidFrozenAmount)?;

                self.frozen_amount = self.get_remaining_quantity() * limit_price
            }
        }

        Ok(())
    }
}
