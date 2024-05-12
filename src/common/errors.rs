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
    OrderIdDuplication
}