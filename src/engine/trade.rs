use crate::common::errors::{AppError, AppResult};

use super::order::{Order, OrderId, OrderPrice, OrderQuantity};

pub type TradeId = u64;

pub struct Trade {
    id: TradeId,
    taker_order_id: OrderId,
    maker_order_id: OrderId,
    price: OrderPrice, 
    quantity: OrderQuantity
}

impl Trade {
    pub fn new(taker_order: &Order, maker_order: &Order, traded_quantity: OrderQuantity) -> AppResult<Self> {
        let price = maker_order
            .get_limit_price()
            .ok_or(AppError::MakerOrderWithoutLimitPrice)?;
        
        Ok(Self {
            id: 0,
            taker_order_id: taker_order.get_id(),
            maker_order_id: maker_order.get_id(),
            price, 
            quantity: traded_quantity
        })
    }
}