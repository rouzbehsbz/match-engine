use std::sync::Arc;

use crate::{balance::{service::{BalanceService, BusinessType, ChangeBalaneInput}, AssetId, BalanceType, UserId}, common::errors::AppResult};

use super::{order::{Order, OrderId, OrderPrice, OrderQuantity, OrderSide}, orderbook::Orderbook, trade::Trade};

pub type PairId = u32;

pub struct Engine {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    orderbook: Orderbook,
    balance_service: Arc<BalanceService>
}

impl Engine {
    pub fn new(base_asset_id: AssetId, quote_asset_id: AssetId, balance_service: Arc<BalanceService>) -> Self {
        Self {
            base_asset_id,
            quote_asset_id,
            orderbook: Orderbook::new(),
            balance_service
        }
    }

    pub fn freeze_user_balance(&self, order: &Order) -> AppResult<()> {
        let remaining_amount = order.get_frozen_amount();

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: order.get_user_id(),
            asset_id: order.get_asset_id(),
            business_type: BusinessType::Trade,
            business_id: 1,
            balance_type: BalanceType::Available,
            amount: -remaining_amount
        })?;

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: order.get_user_id(),
            asset_id: order.get_asset_id(),
            business_type: BusinessType::Trade,
            business_id: 1,
            balance_type: BalanceType::Frozen,
            amount: remaining_amount
        })?;

        Ok(())
    }

    pub fn unfreeze_user_balance(&self, order: &Order) -> AppResult<()> {
        let remaining_amount = order.get_frozen_amount();

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: order.get_user_id(),
            asset_id: order.get_asset_id(),
            business_type: BusinessType::Trade,
            business_id: 1,
            balance_type: BalanceType::Available,
            amount: -remaining_amount
        })?;

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: order.get_user_id(),
            asset_id: order.get_asset_id(),
            business_type: BusinessType::Trade,
            business_id: 1,
            balance_type: BalanceType::Frozen,
            amount: remaining_amount
        })?;

        Ok(())
    }

    pub fn transfer_trade_balance(&self, trade: &Trade) -> AppResult<()> {
        let bid_order = trade.get_bid_order();
        let ask_order = trade.get_ask_order();

        let is_maker_order_bid = match trade.get_maker_order_side() {
            OrderSide::Ask => false,
            OrderSide::Bid => true
        };

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: bid_order.get_user_id(),
            asset_id: bid_order.get_base_asset_id(),
            business_type: BusinessType::Trade,
            business_id: trade.get_id(),
            balance_type: BalanceType::Available,
            amount: trade.get_trade_quantity()
        })?;

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: bid_order.get_user_id(),
            asset_id: bid_order.get_quote_asset_id(),
            business_type: BusinessType::Trade,
            business_id: trade.get_id(),
            balance_type: match is_maker_order_bid {
                true => BalanceType::Frozen,
                false => BalanceType::Available
            },
            amount: -trade.get_trade_amount()
        })?;

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: ask_order.get_user_id(),
            asset_id: ask_order.get_quote_asset_id(),
            business_type: BusinessType::Trade,
            business_id: trade.get_id(),
            balance_type: BalanceType::Available,
            amount: trade.get_trade_amount()
        })?;

        self.balance_service.change_balance(ChangeBalaneInput {
            user_id: ask_order.get_user_id(),
            asset_id: ask_order.get_base_asset_id(),
            business_type: BusinessType::Trade,
            business_id: trade.get_id(),
            balance_type: match is_maker_order_bid {
                true => BalanceType::Available,
                false => BalanceType::Frozen
            },
            amount: -trade.get_trade_quantity()
        })?;

        Ok(())
    }

    pub fn process_new_order(&mut self, user_id: UserId, limit_price: Option<OrderPrice>, quantity: OrderQuantity, side: OrderSide) -> AppResult<()> {
        let order = match limit_price {
            Some(limit_price) => Order::new_limit(user_id, self.base_asset_id, self.quote_asset_id, side, limit_price, quantity),
            None => Order::new_market(user_id, self.base_asset_id, self.quote_asset_id, side, quantity)
        };

        let match_result = self.orderbook.handle_create(order)?;

        for trade in match_result.trades {
            self.transfer_trade_balance(&trade)?;
        }

        if !match_result.taker_order.is_closed() && match_result.taker_order.is_bookable() {
            self.freeze_user_balance(&match_result.taker_order)?;
        }

        for filled_order in match_result.filled_orders {
            self.unfreeze_user_balance(&filled_order)?;
        }

        Ok(())
    }
}