use config::repositories::toml::TomlConfigManager;
use container::Container;
use presentation::grpc::server::{match_engine::trade_server::TradeServer, TradeController};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TomlConfigManager::from_file("./config.test.toml");
    let contaienr = Container::new(&config);

    Server::builder()
        .add_service(TradeServer::new(TradeController::new(
            contaienr.engine_service,
            contaienr.balance_service,
        )))
        .serve("0.0.0.0:3000".parse()?)
        .await?;

    Ok(())
}
