use std::{cmp::Reverse, collections::{btree_map::{Entry, IterMut}, BTreeMap, HashMap, VecDeque}, ops::{Deref, DerefMut}};
use rust_decimal::{prelude::Zero, Decimal};
use crate::common::errors::{AppError, AppResult};
use super::{order::{Order, OrderId, OrderPrice, OrderQuantity, OrderSide}, trade::Trade};

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

        if let Some(index) = self.order_ids.iter().position(|&order_id| order_id == order.get_id()) {
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
                OrderSide::Bid => limit_price >= self.price
            }
            None => true
        }
    }
}

pub type PriceLevelIterMut<'a> = Box<dyn Iterator<Item = &'a mut PriceLevel> + 'a>;

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

    pub fn get_mut_order_by_id(&mut self, order_id: OrderId) -> Option<&mut Order> {
        self
            .orders
            .get_mut(&order_id)
    }

    pub fn get_mut_opposite_price_levels(&mut self, order: &Order) -> PriceLevelIterMut<'_> {
        match order.get_side() {
            OrderSide::Ask => {
                let iter = self.bids.values_mut().map(|price_level| price_level);

                Box::new(iter)
            },
            OrderSide::Bid => {
                let iter = self.asks.values_mut().map(|price_level| price_level);

                Box::new(iter)
            }
        }
    }

    pub fn remove_drained_orderbook_level(&mut self, order: &Order) {
        match order.get_side() {
            OrderSide::Ask => {self.bids.pop_first();},
            OrderSide::Bid => {self.asks.pop_first();}
        };
    }

    pub fn match_order(&mut self, mut taker_order: Order, opposite_price_levels: PriceLevelIterMut<'_>) -> AppResult<()> {
        let mut trades: Vec<Trade> = vec![];
        let mut drained_price_levels = 0;

        for price_level in opposite_price_levels {
            if taker_order.is_closed() || !price_level.is_matches(&taker_order) {
                break;
            }

            let mut total_traded_quantity = Decimal::zero();
            let mut filled_orders = 0;

            for order_id in price_level.order_ids.iter_mut() {
                let maker_order = self.get_mut_order_by_id(*order_id).ok_or(AppError::OrderMatchNotFound)?;

                let traded_quantity = taker_order.get_traded_quantity(&maker_order);

                taker_order.fill(traded_quantity)?;
                maker_order.fill(traded_quantity)?;

                let trade = Trade::new(&taker_order, &maker_order, traded_quantity)?;

                trades.push(trade);

                total_traded_quantity += traded_quantity;

                if maker_order.is_closed() {
                    filled_orders += 1;
                }
            }

            price_level.quantity -= total_traded_quantity;

            for _ in 0..filled_orders {
                price_level.pop_front_order_id()
                    .and_then(|order_id| self.orders.remove(&order_id));
            }

            if price_level.quantity == Decimal::zero() {
                drained_price_levels += 1;
            }
        }

        for _ in 0..drained_price_levels {
            self.remove_drained_orderbook_level(&taker_order)
        }

        Ok(())
    }
}