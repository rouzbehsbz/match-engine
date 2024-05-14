use std::{str::FromStr, sync::Arc};

use rust_decimal::Decimal;
use tonic::{Request, Response};

use crate::{balance::{
    service::{BalanceService, BusinessType},
    BalanceType,
}, engine::{order::{OrderPrice, OrderSide}, service::EngineService}};

use self::match_engine::{
    trade_server::Trade, DepositRequest, DepositResponse, GetMarketOrderbookRequest, GetMarketOrderbookResponse, GetUserBalanceRequest, GetUserBalanceResponse, PlaceOrderRequest, PlaceOrderResponse, PriceLevel, WithdrawRequest, WithdrawResponse
};

use super::GrpcResult;

pub mod match_engine {
    tonic::include_proto!("match_engine");
}

pub struct MatchEngineController {
    engine_service: Arc<EngineService>,
    balance_service: Arc<BalanceService>,
}

impl MatchEngineController {
    pub fn new(engine_service: Arc<EngineService>, balance_service: Arc<BalanceService>) -> Self {
        Self { engine_service, balance_service }
    }
}

#[tonic::async_trait]
impl Trade for MatchEngineController {
    async fn get_user_balance(
        &self,
        request: Request<GetUserBalanceRequest>,
    ) -> GrpcResult<GetUserBalanceResponse> {
        let request = request.into_inner();

        let balance_status = self
            .balance_service
            .get_balance_status(request.user_id, request.asset_id);

        let response = GetUserBalanceResponse {
            total: balance_status.total.to_string(),
            available: balance_status.available.to_string(),
            frozen: balance_status.frozen.to_string(),
        };

        Ok(Response::new(response))
    }

    async fn withdraw(
        &self,
        request: Request<WithdrawRequest>,
    ) -> GrpcResult<WithdrawResponse> {
        let request = request.into_inner();
        let amount = Decimal::from_str(&request.amount).unwrap();

        self.balance_service.change_balance(
            request.user_id,
            request.asset_id,
            BusinessType::Withdraw,
            1,
            BalanceType::Available,
            amount,
        ).unwrap();

        Ok(Response::new(WithdrawResponse {  }))
    }

    async fn deposit(
        &self,
        request: Request<DepositRequest>,
    ) -> GrpcResult<DepositResponse> {
        let request = request.into_inner();
        let amount = Decimal::from_str(&request.amount).unwrap();

        self.balance_service.change_balance(
            request.user_id,
            request.asset_id,
            BusinessType::Deposit,
            1,
            BalanceType::Available,
            -amount,
        ).unwrap();

        Ok(Response::new(DepositResponse {  }))
    }

    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> GrpcResult<PlaceOrderResponse> {
        let request = request.into_inner();

        let limit_price: Option<OrderPrice> = match request.limit_price.is_empty() {
            true => None,
            false => Some(Decimal::from_str(&request.limit_price).unwrap())
        };
        let quantity = Decimal::from_str(&request.quantity).unwrap();

        let order_side = match request.side {
            0 => OrderSide::Ask,
            1 => OrderSide::Bid,
            _ => OrderSide::Ask
        };

        self.engine_service.place_order(request.pair_id, request.user_id, limit_price, quantity, order_side).unwrap();
    
        Ok(Response::new(PlaceOrderResponse { order_id: 0 }))
    }

    async fn get_market_orderbook(
        &self,
        request: Request<GetMarketOrderbookRequest>,
    ) -> GrpcResult<GetMarketOrderbookResponse> {
        let request = request.into_inner();

        let (asks_depth, bids_depth) = self.engine_service.get_market_orderbook(request.pair_id);

        let asks: Vec<PriceLevel> = asks_depth.iter().map(|value| {
            PriceLevel { price: value[0].to_string(), quantity: value[1].to_string() }
        }).collect();

        let bids: Vec<PriceLevel> = bids_depth.iter().map(|value| {
            PriceLevel { price: value[0].to_string(), quantity: value[1].to_string() }
        }).collect();

        let response = GetMarketOrderbookResponse {
            asks,
            bids
        };

        Ok(Response::new(response))
    }
}
