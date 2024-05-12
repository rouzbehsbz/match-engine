use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("User doesn't have enough balance.")]
    UserBalanceExceeds
}