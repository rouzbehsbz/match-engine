pub trait BalanceEventExector {
    fn push_balance(&self);
    fn push_withdraw(&self);
    fn push_deposit(&self);
}

pub trait TradeEventExector {
    fn push_order(&self);
    fn push_trade(&self);
}