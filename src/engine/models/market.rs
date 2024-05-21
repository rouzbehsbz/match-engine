use std::sync::Arc;

use crate::{
    balance::{
        service::{BalanceService, BusinessType},
        AssetId, BalanceType, UserId,
    },
    common::{
        errors::{AppError, AppResult},
        sequencer::Sequencer,
    },
};

use super::{
    order::{Order, OrderPrice, OrderQuantity, OrderSide},
    orderbook::{Orderbook, OrderbookDepth},
    trade::Trade,
};

pub type PairId = u32;

pub struct Market {
    base_asset_id: AssetId,
    quote_asset_id: AssetId,
    is_market_trade_enabled: bool,
    min_allowed_quantity: OrderQuantity,

    orderbook: Orderbook,
    order_id_sequencer: Arc<Sequencer>,
    balance_service: Arc<BalanceService>,
}

impl Market {
    pub fn new(
        base_asset_id: AssetId,
        quote_asset_id: AssetId,
        is_market_trade_enabled: bool,
        min_allowed_quantity: OrderQuantity,
        balance_service: Arc<BalanceService>,
        order_id_sequencer: Arc<Sequencer>,
    ) -> Self {
        Self {
            base_asset_id,
            quote_asset_id,
            orderbook: Orderbook::new(),
            balance_service,

            is_market_trade_enabled,
            min_allowed_quantity,
            order_id_sequencer,
        }
    }

    pub fn freeze_user_balance(&self, order: &Order) -> AppResult<()> {
        let remaining_amount = order.get_frozen_amount();

        self.balance_service.change_balance(
            order.get_user_id(),
            order.get_asset_id(),
            BusinessType::Trade,
            1,
            BalanceType::Available,
            -remaining_amount,
        )?;

        self.balance_service.change_balance(
            order.get_user_id(),
            order.get_asset_id(),
            BusinessType::Trade,
            1,
            BalanceType::Frozen,
            remaining_amount,
        )?;

        Ok(())
    }

    pub fn unfreeze_user_balance(&self, order: &Order) -> AppResult<()> {
        let remaining_amount = order.get_frozen_amount();

        self.balance_service.change_balance(
            order.get_user_id(),
            order.get_asset_id(),
            BusinessType::Trade,
            1,
            BalanceType::Available,
            -remaining_amount,
        )?;

        self.balance_service.change_balance(
            order.get_user_id(),
            order.get_asset_id(),
            BusinessType::Trade,
            1,
            BalanceType::Frozen,
            remaining_amount,
        )?;

        Ok(())
    }

    pub fn transfer_trade_balance(&self, trade: &Trade) -> AppResult<()> {
        let bid_order = trade.get_bid_order();
        let ask_order = trade.get_ask_order();

        let is_maker_order_bid = match trade.get_maker_order_side() {
            OrderSide::Ask => false,
            OrderSide::Bid => true,
        };

        self.balance_service.change_balance(
            bid_order.get_user_id(),
            bid_order.get_base_asset_id(),
            BusinessType::Trade,
            trade.get_id(),
            BalanceType::Available,
            trade.get_quantity(),
        )?;

        self.balance_service.change_balance(
            bid_order.get_user_id(),
            bid_order.get_quote_asset_id(),
            BusinessType::Trade,
            trade.get_id(),
            match is_maker_order_bid {
                true => BalanceType::Frozen,
                false => BalanceType::Available,
            },
            -trade.get_amount(),
        )?;

        self.balance_service.change_balance(
            ask_order.get_user_id(),
            ask_order.get_quote_asset_id(),
            BusinessType::Trade,
            trade.get_id(),
            BalanceType::Available,
            trade.get_amount(),
        )?;

        self.balance_service.change_balance(
            ask_order.get_user_id(),
            ask_order.get_base_asset_id(),
            BusinessType::Trade,
            trade.get_id(),
            match is_maker_order_bid {
                true => BalanceType::Available,
                false => BalanceType::Frozen,
            },
            -trade.get_quantity(),
        )?;

        Ok(())
    }

    pub fn check_new_order_input(&self, order: &Order) -> AppResult<()> {
        if order.get_limit_price().is_none() && !self.is_market_trade_enabled {
            return Err(AppError::MarketTradeDisbaled);
        }

        if order.get_quantity().lt(&self.min_allowed_quantity) {
            return Err(AppError::MarketMinimumAllowedQuantityExceeds);
        }

        if let Some(limit_price) = order.get_limit_price() {
            if limit_price.is_zero() {
                return Err(AppError::LimitOrderInvalidPrice);
            }
        } else {
            match order.get_side() {
                OrderSide::Ask => {
                    if self.orderbook.is_bids_empty() {
                        return Err(AppError::CounterOrderbooksIsEmpty);
                    }
                }
                OrderSide::Bid => {
                    if self.orderbook.is_asks_empty() {
                        return Err(AppError::CounterOrderbooksIsEmpty);
                    }
                }
            }
        }

        match order.get_side() {
            OrderSide::Ask => {
                if !self.balance_service.is_available_balance_enough(
                    order.get_user_id(),
                    order.get_base_asset_id(),
                    order.get_quantity(),
                ) {
                    return Err(AppError::UserBalanceExceeds);
                }
            }
            OrderSide::Bid => {
                if !self.balance_service.is_available_balance_enough(
                    order.get_user_id(),
                    order.get_quote_asset_id(),
                    order.get_amount()?,
                ) {
                    return Err(AppError::UserBalanceExceeds);
                }
            }
        }

        Ok(())
    }

    pub fn process_new_order(
        &mut self,
        user_id: UserId,
        limit_price: Option<OrderPrice>,
        quantity: OrderQuantity,
        side: OrderSide,
    ) -> AppResult<()> {
        let order = match limit_price {
            Some(limit_price) => Order::new_limit(
                self.order_id_sequencer.next(),
                user_id,
                self.base_asset_id,
                self.quote_asset_id,
                side,
                limit_price,
                quantity,
            ),
            None => Order::new_market(
                self.order_id_sequencer.next(),
                user_id,
                self.base_asset_id,
                self.quote_asset_id,
                side,
                quantity,
            ),
        };

        self.check_new_order_input(&order)?;

        let match_result = self.orderbook.put_order(order)?;

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

    pub fn get_orderbook_depth(&self) -> (OrderbookDepth, OrderbookDepth) {
        let asks_depth = self.orderbook.get_asks_depth();
        let bids_depth = self.orderbook.get_bids_depth();

        (asks_depth, bids_depth)
    }
}
