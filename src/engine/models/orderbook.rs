use super::{
    order::{Order, OrderId, OrderPrice, OrderQuantity, OrderSide},
    trade::Trade,
};
use crate::common::errors::{AppError, AppResult};
use rust_decimal::{prelude::Zero, Decimal};
use std::{
    cmp::Reverse,
    collections::{btree_map::Entry, BTreeMap, HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

pub struct OrderbookWrapper<T>(T);

impl<T> Deref for OrderbookWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OrderbookWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OrderbookWrapper<BTreeMap<OrderPrice, PriceLevel>> {
    pub fn insert(&mut self, order: &Order) -> AppResult<()> {
        let limit_price = order
            .get_limit_price()
            .ok_or(AppError::OrderbookInsertWithNoLimitPrice)?;

        let price_level = self
            .0
            .entry(limit_price)
            .or_insert_with(|| PriceLevel::new(limit_price));

        price_level.insert(order);

        Ok(())
    }

    pub fn remove(&mut self, order: &Order) -> AppResult<()> {
        let limit_price = order
            .get_limit_price()
            .ok_or(AppError::OrderbookRemoveWithNoLimitPrice)?;

        let Entry::Occupied(mut price_level) = self.0.entry(limit_price) else {
            unreachable!();
        };

        if price_level.get().order_ids.len() == 1 {
            price_level.remove();
        } else {
            let price_level = price_level.get_mut();

            price_level.remove(order);
        }

        Ok(())
    }
}

impl OrderbookWrapper<BTreeMap<Reverse<OrderPrice>, PriceLevel>> {
    pub fn insert(&mut self, order: &Order) -> AppResult<()> {
        let limit_price = order
            .get_limit_price()
            .ok_or(AppError::OrderbookInsertWithNoLimitPrice)?;

        let price_level = self
            .0
            .entry(Reverse(limit_price))
            .or_insert_with(|| PriceLevel::new(limit_price));

        price_level.insert(order);

        Ok(())
    }

    pub fn remove(&mut self, order: &Order) -> AppResult<()> {
        let limit_price = order
            .get_limit_price()
            .ok_or(AppError::OrderbookRemoveWithNoLimitPrice)?;

        let Entry::Occupied(mut price_level) = self.0.entry(Reverse(limit_price)) else {
            unreachable!();
        };

        if price_level.get().order_ids.len() == 1 {
            price_level.remove();
        } else {
            let price_level = price_level.get_mut();

            price_level.remove(order);
        }

        Ok(())
    }
}

pub struct PriceLevel {
    order_ids: VecDeque<OrderId>,
    quantity: OrderQuantity,
    price: OrderPrice,
}

impl PriceLevel {
    pub fn new(price: OrderPrice) -> Self {
        Self {
            order_ids: VecDeque::new(),
            quantity: Decimal::zero(),
            price,
        }
    }

    pub fn insert(&mut self, order: &Order) {
        self.quantity += order.get_remaining_quantity();
        self.order_ids.push_back(order.get_id());
    }

    pub fn remove(&mut self, order: &Order) {
        self.quantity -= order.get_remaining_quantity();

        if let Some(index) = self
            .order_ids
            .iter()
            .position(|&order_id| order_id == order.get_id())
        {
            self.order_ids.remove(index);
        }
    }

    pub fn pop_front_order_id(&mut self) -> Option<OrderId> {
        self.order_ids.pop_front()
    }

    pub fn is_closed(&self) -> bool {
        self.quantity == Decimal::zero()
    }

    pub fn is_matches(&self, order: &Order) -> bool {
        if self.is_closed() || order.is_closed() {
            return false;
        }

        match order.get_limit_price() {
            Some(limit_price) => match order.get_side() {
                OrderSide::Ask => limit_price <= self.price,
                OrderSide::Bid => limit_price >= self.price,
            },
            None => true,
        }
    }
}

pub type AsksOrderbook = OrderbookWrapper<BTreeMap<OrderPrice, PriceLevel>>;
pub type BidsOrderbook = OrderbookWrapper<BTreeMap<Reverse<OrderPrice>, PriceLevel>>;
pub type OrdersIndex = HashMap<OrderId, Order>;

pub struct Orderbook {
    asks: AsksOrderbook,
    bids: BidsOrderbook,
    orders: OrdersIndex,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            asks: OrderbookWrapper(BTreeMap::new()),
            bids: OrderbookWrapper(BTreeMap::new()),
            orders: HashMap::new(),
        }
    }

    pub fn remove_drained_orderbook_level(&mut self, order: &Order) {
        match order.get_side() {
            OrderSide::Ask => {
                self.bids.pop_first();
            }
            OrderSide::Bid => {
                self.asks.pop_first();
            }
        };
    }

    pub fn match_bid_order(&mut self, mut taker_order: Order) -> AppResult<MatchOrderOutput> {
        let mut trades: Vec<Trade> = vec![];
        let mut filled_orders: Vec<Order> = vec![];
        let mut drained_price_levels = 0;

        for (_, price_level) in self.asks.iter_mut() {
            if taker_order.is_closed() || !price_level.is_matches(&taker_order) {
                break;
            }

            let mut total_traded_quantity = Decimal::zero();
            let mut filled_orders_count = 0;

            for order_id in price_level.order_ids.iter_mut() {
                let maker_order = self
                    .orders
                    .get_mut(&order_id)
                    .ok_or(AppError::OrderMatchNotFound)?;

                let traded_quantity = taker_order.get_traded_quantity(&maker_order);

                taker_order.fill(traded_quantity)?;
                maker_order.fill(traded_quantity)?;

                let trade = Trade::new(&taker_order, &maker_order, traded_quantity)?;

                trades.push(trade);

                total_traded_quantity += traded_quantity;

                maker_order.decrease_frozen_amount(traded_quantity)?;

                if maker_order.is_closed() {
                    filled_orders_count += 1;
                    filled_orders.push(maker_order.clone());
                }
            }

            price_level.quantity -= total_traded_quantity;

            for _ in 0..filled_orders_count {
                price_level
                    .pop_front_order_id()
                    .and_then(|order_id| self.orders.remove(&order_id));
            }

            if price_level.quantity == Decimal::zero() {
                drained_price_levels += 1;
            }
        }

        for _ in 0..drained_price_levels {
            self.remove_drained_orderbook_level(&taker_order)
        }

        if !taker_order.is_closed() && taker_order.is_bookable() {
            taker_order.set_frozen_amount()?;

            self.bids.insert(&taker_order)?;
            self.orders.insert(taker_order.get_id(), taker_order);
        }

        Ok(MatchOrderOutput {
            taker_order: taker_order.clone(),
            filled_orders,
            trades,
        })
    }

    pub fn match_ask_order(&mut self, mut taker_order: Order) -> AppResult<MatchOrderOutput> {
        let mut trades: Vec<Trade> = vec![];
        let mut filled_orders: Vec<Order> = vec![];
        let mut drained_price_levels = 0;

        for (_, price_level) in self.bids.iter_mut() {
            if taker_order.is_closed() || !price_level.is_matches(&taker_order) {
                break;
            }

            let mut total_traded_quantity = Decimal::zero();
            let mut filled_orders_count = 0;

            for order_id in price_level.order_ids.iter_mut() {
                let maker_order = self
                    .orders
                    .get_mut(&order_id)
                    .ok_or(AppError::OrderMatchNotFound)?;

                let traded_quantity = taker_order.get_traded_quantity(&maker_order);

                taker_order.fill(traded_quantity)?;
                maker_order.fill(traded_quantity)?;

                let trade = Trade::new(&taker_order, &maker_order, traded_quantity)?;

                trades.push(trade);

                total_traded_quantity += traded_quantity;

                maker_order.decrease_frozen_amount(traded_quantity)?;

                if maker_order.is_closed() {
                    filled_orders_count += 1;
                    filled_orders.push(maker_order.clone())
                }
            }

            price_level.quantity -= total_traded_quantity;

            for _ in 0..filled_orders_count {
                price_level
                    .pop_front_order_id()
                    .and_then(|order_id| self.orders.remove(&order_id));
            }

            if price_level.quantity == Decimal::zero() {
                drained_price_levels += 1;
            }
        }

        for _ in 0..drained_price_levels {
            self.remove_drained_orderbook_level(&taker_order)
        }

        if !taker_order.is_closed() && taker_order.is_bookable() {
            taker_order.set_frozen_amount()?;

            self.bids.insert(&taker_order)?;
            self.orders.insert(taker_order.get_id(), taker_order);
        }

        Ok(MatchOrderOutput {
            taker_order: taker_order.clone(),
            filled_orders,
            trades,
        })
    }

    pub fn handle_create(&mut self, order: Order) -> AppResult<MatchOrderOutput> {
        // if self.orders.contains_key(&order.get_id()) {
        //     return Err(AppError::OrderIdDuplication)
        // }

        match order.get_side() {
            OrderSide::Ask => {
                let match_result = self.match_ask_order(order)?;

                Ok(match_result)
            }
            OrderSide::Bid => {
                let match_result = self.match_bid_order(order)?;

                Ok(match_result)
            }
        }
    }

    pub fn handle_cancel(&mut self, order_id: OrderId) -> AppResult<()> {
        let order = self
            .orders
            .remove(&order_id)
            .ok_or(AppError::OrderIdNotFound)?;

        match order.get_side() {
            OrderSide::Ask => self.asks.remove(&order)?,
            OrderSide::Bid => self.bids.remove(&order)?,
        }

        Ok(())
    }

    pub fn is_asks_empty(&self) -> bool {
        self.asks.is_empty()
    }

    pub fn is_bids_empty(&self) -> bool {
        self.bids.is_empty()
    }

    pub fn get_asks_depth(&self) -> OrderbookDepth {
        let depth: Vec<[Decimal; 2]> = self.asks.iter().map(|(price, level)| {
            [price.clone(), level.quantity.clone()]
        }).collect();

        depth
    }

    pub fn get_bids_depth(&self) -> OrderbookDepth {
        let depth: Vec<[Decimal; 2]> = self.bids.iter().map(|(price, level)| {
            [price.0.clone(), level.quantity.clone()]
        }).collect();

        depth
    }
}

pub type OrderbookDepth = Vec<[Decimal; 2]>;

pub struct MatchOrderOutput {
    pub taker_order: Order,
    pub filled_orders: Vec<Order>,
    pub trades: Vec<Trade>,
}
