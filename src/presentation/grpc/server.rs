use std::sync::Arc;

use tonic::{Request, Response};

use crate::{balance::{
    service::{BalanceService, BusinessType},
    BalanceType,
}, engine::service::EngineService};

use self::match_engine::{
    trade_server::Trade,
    WithdrawRequest, WithdrawResponse,
    DepositRequest, DepositResponse,
    GetMarketOrderbookRequest, GetMarketOrderbookResponse, GetUserBalanceRequest,
    GetUserBalanceResponse, PlaceOrderRequest, PlaceOrderResponse,
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
            total: balance_status.total,
            available: balance_status.available,
            frozen: balance_status.frozen,
        };

        Ok(Response::new(response))
    }

    async fn withdraw(
        &self,
        request: Request<WithdrawRequest>,
    ) -> GrpcResult<WithdrawResponse> {
        let request = request.into_inner();

        self.balance_service.change_balance(
            request.user_id,
            request.asset_id,
            BusinessType::Withdraw,
            1,
            BalanceType::Available,
            request.amount,
        ).unwrap();

        Ok(Response::new(WithdrawResponse {  }))
    }

    async fn deposit(
        &self,
        request: Request<DepositRequest>,
    ) -> GrpcResult<DepositResponse> {
        let request = request.into_inner();

        self.balance_service.change_balance(
            request.user_id,
            request.asset_id,
            BusinessType::Deposit,
            1,
            BalanceType::Available,
            -request.amount,
        ).unwrap();

        Ok(Response::new(DepositResponse {  }))
    }

    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> GrpcResult<PlaceOrderResponse> {
        todo!()
    }

    async fn get_market_orderbook(
        &self,
        request: Request<GetMarketOrderbookRequest>,
    ) -> GrpcResult<GetMarketOrderbookResponse> {
        let request = request.into_inner();

        let (asks_depth, bids_depth) = self.engine_service.get_market_orderbook(request.pair_id);

        let response = GetMarketOrderbookResponse {
            asks: asks_depth,
            bids: bids_depth
        };

        Ok(Response::new(response))
    }
}
