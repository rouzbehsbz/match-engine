use rust_decimal::{prelude::Zero, Decimal};

use crate::common::errors::{AppError, AppResult};

pub type OrderId = u64;
pub type OrderPrice = Decimal;
pub type OrderQuantity = Decimal;

pub enum OrderType {
    Limit {
        price: OrderPrice
    },
    Market
}

#[derive(Clone, Copy)]
pub enum OrderSide {
    Ask,
    Bid
}

pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Cancelled,
    Closed,
    Filled
}

pub struct Order {
    id: OrderId,
    type_: OrderType,
    side: OrderSide,
    quantity: Decimal,
    filled_quantity: Decimal,
    status: OrderStatus
}

impl Order {
    pub fn get_id(&self) -> OrderId {
        self.id
    }

    pub fn get_limit_price(&self) -> Option<OrderPrice> {
        match self.type_ {
            OrderType::Limit { price } => Some(price),
            OrderType::Market => None
        }
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn get_remaining_quantity(&self) -> OrderQuantity {
        self.quantity - self.filled_quantity
    }

    pub fn get_traded_quantity(&self, matched_order: &Order) -> OrderQuantity {
        self.get_remaining_quantity().min(matched_order.get_remaining_quantity())
    }

    pub fn fill(&mut self, quantity: OrderQuantity) -> AppResult<()> {
        if quantity > self.get_remaining_quantity() {
            return Err(AppError::OrderOverFilled);
        }

        self.quantity += quantity;
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
            _ => false
        }
    }

    pub fn is_bookable(&self) -> bool {
        match self.type_ {
            OrderType::Limit { .. } => true,
            OrderType::Market { .. } => false,
        }
    }
}