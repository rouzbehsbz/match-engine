use balance::{service::BusinessType, BalanceType, UserId};
use config::repositories::toml::TomlConfigManager;
use container::Container;
use engine::models::order::OrderSide;
use presentation::grpc::server::{match_engine::trade_server::TradeServer, TradeController};
use rust_decimal::Decimal;
use tonic::transport::Server;

pub mod balance;
pub mod common;
pub mod config;
pub mod container;
pub mod engine;
pub mod presentation;

//TODO: handle asset precisions
//TODO: handle trade fees
//TODO: handle event sourcing
//TODO: handle TIF for orderbook

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TomlConfigManager::from_file("./config.test.toml");
    let container = Container::new(&config);

    let Rouzbeh: UserId = 1;
    let Kasra: UserId = 2;

    let BTC = 1;
    let USDT = 2;

    container.balance_service.change_balance(Rouzbeh, USDT, BusinessType::Deposit, 1, BalanceType::Available, Decimal::from(1000))?;
    container.balance_service.change_balance(Kasra, BTC, BusinessType::Deposit, 1, BalanceType::Available, Decimal::from(1000))?;

    container.engine_service.place_order(1, Rouzbeh, Some(Decimal::from(20)), Decimal::from(10), OrderSide::Bid)?;
    container.engine_service.place_order(1, Rouzbeh, Some(Decimal::from(17)), Decimal::from(5), OrderSide::Bid)?;
    container.engine_service.place_order(1, Kasra, Some(Decimal::from(18)), Decimal::from(50), OrderSide::Ask)?;

    let ob = container.engine_service.get_market_orderbook(1);

    println!("{:?}", ob);

    println!("Rouzbeh BTC {:?}", container.balance_service.get_balance_status(Rouzbeh, BTC));
    println!("Rouzbeh USDT {:?}", container.balance_service.get_balance_status(Rouzbeh, USDT));
    println!("----");
    println!("Kasra BTC {:?}", container.balance_service.get_balance_status(Kasra, BTC));
    println!("Kasra USDT {:?}", container.balance_service.get_balance_status(Kasra, USDT));

    // Server::builder()
    // .add_service(TradeServer::new(TradeController::new(
    //     container.engine_service,
    //     container.balance_service,
    // )))
    // .serve("0.0.0.0:3000".parse()?)
    // .await?;

    Ok(())
}
