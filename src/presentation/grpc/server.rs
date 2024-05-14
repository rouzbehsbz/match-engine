use std::sync::Arc;

use tonic::{Request, Response};

use crate::balance::service::BalanceService;

use self::match_engine::{trade_server::Trade, ChangeUserBalanceRequest, ChangeUserBalanceResponse, GetMarketOrderbookRequest, GetMarketOrderbookResponse, GetUserBalanceRequest, GetUserBalanceResponse, PlaceOrderRequest, PlaceOrderResponse};

use super::GrpcResult;

pub mod match_engine {
    tonic::include_proto!("match_engine");
}

pub struct MatchEngineController {
    balance_service: Arc<BalanceService>
}

impl MatchEngineController {
    pub fn new(balance_service: Arc<BalanceService>) -> Self {
        Self {
            balance_service
        }
    }
}

#[tonic::async_trait]
impl Trade for MatchEngineController {
    async fn get_user_balance(&self, request: Request<GetUserBalanceRequest>) -> GrpcResult<GetUserBalanceResponse> {
        let request = request.into_inner();
        
        let balance_status = self.balance_service.get_balance_status(request.user_id, request.asset_id);

        let response = GetUserBalanceResponse {
            total: balance_status.total,
            available: balance_status.available,
            frozen: balance_status.frozen
        };

        Ok(Response::new(response))
    }

    async fn change_user_balance(&self, request: Request<ChangeUserBalanceRequest>) -> GrpcResult<ChangeUserBalanceResponse> {
        todo!()
    }

    async fn place_order(&self, request: Request<PlaceOrderRequest>) -> GrpcResult<PlaceOrderResponse> {
        todo!()
    }

    async fn get_market_orderbook(&self, request: Request<GetMarketOrderbookRequest>) -> GrpcResult<GetMarketOrderbookResponse> {
        todo!()
    }
}