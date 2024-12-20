use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("User doesn't have enough balance.")]
    UserBalanceExceeds,

    #[error("Can't insert order with no limit price in orderbook.")]
    OrderbookInsertWithNoLimitPrice,

    #[error("Can't remove order with no limit price in orderbook.")]
    OrderbookRemoveWithNoLimitPrice,

    #[error("Matched maker order doesn't exists for this order.")]
    OrderMatchNotFound,

    #[error("Maker order doesn't have limit price.")]
    MakerOrderWithoutLimitPrice,

    #[error("Order is over filled.")]
    OrderOverFilled,

    #[error("Order with exact same ID is already booked.")]
    OrderIdDuplication,

    #[error("Order with this ID does'nt exists.")]
    OrderIdNotFound,

    #[error("Order amount without limit price can't be freeze or unfreeze.")]
    OrderInavlidFrozenAmount,

    #[error("Market trade is disabled for this pair.")]
    MarketTradeDisbaled,

    #[error("Order quantity must be greater than minimum market quantity amount.")]
    MarketMinimumAllowedQuantityExceeds,

    #[error("Order limit price is invalid.")]
    LimitOrderInvalidPrice,

    #[error("Counter orderbook is empty.")]
    CounterOrderbooksIsEmpty,

    #[error("Order amount for makrket order is invalid.")]
    InvalidMarketOrderAmount,

    #[error("Market with this pair ID does'nt found.")]
    MarketNotFound,
}
