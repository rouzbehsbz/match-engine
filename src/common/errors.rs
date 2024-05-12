use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("User doesn't have enough balance.")]
    UserBalanceExceeds,

    #[error("Can't insert order with no limit price in orderbook.")]
    OrderbookInsertWithNoLimitPrice,

    #[error("Can't remove order with no limit price in orderbook.")]
    OrderbookRemoveWithNoLimitPrice
}