use rust_decimal::Decimal;

pub type OrderId = u64;
pub type OrderPrice = Decimal;
pub type OrderQuantity = Decimal;

pub enum OrderType {
    Limit {
        price: OrderPrice
    },
    Market
}

pub struct Order {
    id: OrderId,
    type_: OrderType,
    quantity: Decimal,
    filled_quantity: Decimal
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

    pub fn get_remaining_quantity(&self) -> OrderQuantity {
        self.quantity - self.filled_quantity
    }
}