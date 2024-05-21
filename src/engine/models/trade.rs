use rust_decimal::Decimal;

use crate::common::errors::{AppError, AppResult};

use super::order::{Order, OrderPrice, OrderQuantity, OrderSide};

pub type TradeId = u64;

#[derive(Debug)]
pub struct Trade {
    id: TradeId,
    taker_order: Order,
    maker_order: Order,
    price: OrderPrice,
    quantity: OrderQuantity,
}

impl Trade {
    pub fn new(
        taker_order: &Order,
        maker_order: &Order,
        traded_quantity: OrderQuantity,
    ) -> AppResult<Self> {
        let price = maker_order
            .get_limit_price()
            .ok_or(AppError::MakerOrderWithoutLimitPrice)?;

        Ok(Self {
            id: 0,
            taker_order: taker_order.clone(),
            maker_order: maker_order.clone(),
            price,
            quantity: traded_quantity,
        })
    }

    pub fn get_id(&self) -> TradeId {
        self.id
    }

    pub fn get_price(&self) -> OrderPrice {
        self.price
    }

    pub fn get_maker_order_side(&self) -> OrderSide {
        self.maker_order.get_side()
    }

    pub fn get_bid_order(&self) -> Order {
        match self.taker_order.get_side() {
            OrderSide::Ask => self.maker_order,
            OrderSide::Bid => self.taker_order,
        }
    }

    pub fn get_ask_order(&self) -> Order {
        match self.taker_order.get_side() {
            OrderSide::Ask => self.taker_order,
            OrderSide::Bid => self.maker_order,
        }
    }

    pub fn get_quantity(&self) -> OrderQuantity {
        self.quantity
    }

    pub fn get_amount(&self) -> Decimal {
        self.quantity * self.price
    }
}
