use std::{cmp::Reverse, collections::{btree_map::Entry, BTreeMap, HashMap, VecDeque}, ops::{Deref, DerefMut}};
use rust_decimal::{prelude::Zero, Decimal};
use crate::common::errors::{AppError, AppResult};
use super::order::{Order, OrderId, OrderPrice, OrderQuantity};

pub struct OrderbookWrapper<T>(T);

impl <T> Deref for OrderbookWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <T> DerefMut for OrderbookWrapper<T> {
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
        }
        else {
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
        }
        else {
            let price_level = price_level.get_mut();

            price_level.remove(order);
        }

        Ok(())
    }
}

pub struct PriceLevel {
    order_ids: VecDeque<OrderId>,
    quantity: OrderQuantity,
    price: OrderPrice
}

impl PriceLevel {
    pub fn new(price: OrderPrice) -> Self {
        Self {
            order_ids: VecDeque::new(),
            quantity: Decimal::zero(),
            price
        }
    }

    pub fn insert(&mut self, order: &Order) {
        self.quantity += order.get_remaining_quantity();
        self.order_ids.push_back(order.get_id());
    }

    pub fn remove(&mut self, order: &Order) {
        self.quantity -= order.get_remaining_quantity();

        if let Some(index) = self.order_ids.iter().position(|&order_id| order.get_id() == order_id) {
            self.order_ids.remove(index);
        } 
    }
}

pub type AsksOrderbook = OrderbookWrapper<BTreeMap<OrderPrice, PriceLevel>>;
pub type BidsOrderbook = OrderbookWrapper<BTreeMap<Reverse<OrderPrice>, PriceLevel>>;
pub type OrdersIndex = HashMap<OrderId, Order>;

pub struct Orderbook {
    asks: AsksOrderbook,
    bids: BidsOrderbook,
    orders: OrdersIndex
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            asks: OrderbookWrapper(BTreeMap::new()),
            bids: OrderbookWrapper(BTreeMap::new()),
            orders: HashMap::new()
        }
    }
}