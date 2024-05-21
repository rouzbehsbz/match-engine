use balance::{service::BusinessType, BalanceType, UserId};
use config::repositories::toml::TomlConfigManager;
use container::Container;
use engine::models::order::OrderSide;
use presentation::grpc::server::{match_engine::trade_server::TradeServer, TradeController};
use rust_decimal::Decimal;
use tonic::transport::Server;

pub mod __tests__;
pub mod balance;
pub mod common;
pub mod config;
pub mod container;
pub mod engine;
pub mod presentation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3000".parse()?;

    let config = &TomlConfigManager::from_file("config.test.toml");
    let container = Container::new(config);

    let trade_controller =
        TradeController::new(container.engine_service, container.balance_service);

    Server::builder()
        .add_service(TradeServer::new(trade_controller))
        .serve(addr)
        .await?;

    Ok(())
}
