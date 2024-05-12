use crate::{balance::UserId, common::errors::AppResult};

use super::{order::{Order, OrderId, OrderPrice, OrderQuantity, OrderSide}, orderbook::Orderbook};

pub type PairId = u32;

pub struct EngineService {
    orderbook: Orderbook
}

pub enum ProcessOrderInput {
    Create {
        user_id: UserId,
        pair_id: PairId,
        limit_price: Option<OrderPrice>,
        quantity: OrderQuantity,
        side: OrderSide,
    },
    Cancel {
        order_id: OrderId
    }
}

impl EngineService {
    pub fn new() -> Self {
        Self {
            orderbook: Orderbook::new()
        }
    }

    pub fn process_order(&mut self, input: ProcessOrderInput) -> AppResult<()> {
        match input {
            ProcessOrderInput::Create {
                user_id,
                pair_id,
                limit_price,
                quantity,
                side,
            } => {
                let order = match limit_price {
                    Some(limit_price) => Order::new_limit(side, limit_price, quantity),
                    None => Order::new_market(side, quantity)
                };

                let _ = self.orderbook.handle_create(order)?;
            },
            ProcessOrderInput::Cancel { 
                order_id 
            } => {
                self.orderbook.handle_cancel(order_id)?;
            }
        }

        Ok(())
    }
}