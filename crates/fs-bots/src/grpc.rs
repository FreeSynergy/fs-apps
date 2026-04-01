// grpc.rs — gRPC service implementation for fs-bots.

use tonic::{Request, Response, Status};

use crate::controller::BotController;
use crate::model::MessagingBot;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("bots");
}

pub use proto::bots_service_server::{BotsService, BotsServiceServer};
pub use proto::{
    BotProto, DisableBotRequest, DisableBotResponse, EnableBotRequest, EnableBotResponse,
    GetBotRequest, GetBotResponse, HealthRequest, HealthResponse, ListBotsRequest,
    ListBotsResponse,
};

fn to_proto(bot: &MessagingBot) -> BotProto {
    BotProto {
        id: bot.id.clone(),
        name: bot.name.clone(),
        kind_label: bot.kind.label().to_owned(),
        enabled: bot.enabled,
    }
}

/// gRPC service backed by a shared [`BotController`].
pub struct GrpcBotsApp {
    ctrl: BotController,
}

impl GrpcBotsApp {
    #[must_use]
    pub fn new(ctrl: BotController) -> Self {
        Self { ctrl }
    }
}

#[tonic::async_trait]
impl BotsService for GrpcBotsApp {
    async fn list_bots(
        &self,
        _req: Request<ListBotsRequest>,
    ) -> Result<Response<ListBotsResponse>, Status> {
        let bots = self.ctrl.list().iter().map(to_proto).collect();
        Ok(Response::new(ListBotsResponse { bots }))
    }

    async fn get_bot(
        &self,
        req: Request<GetBotRequest>,
    ) -> Result<Response<GetBotResponse>, Status> {
        let id = req.into_inner().id;
        match self.ctrl.get(&id) {
            Some(bot) => Ok(Response::new(GetBotResponse {
                bot: Some(to_proto(&bot)),
                found: true,
            })),
            None => Ok(Response::new(GetBotResponse {
                bot: None,
                found: false,
            })),
        }
    }

    async fn enable_bot(
        &self,
        req: Request<EnableBotRequest>,
    ) -> Result<Response<EnableBotResponse>, Status> {
        let ok = self.ctrl.enable(&req.into_inner().id);
        Ok(Response::new(EnableBotResponse { ok }))
    }

    async fn disable_bot(
        &self,
        req: Request<DisableBotRequest>,
    ) -> Result<Response<DisableBotResponse>, Status> {
        let ok = self.ctrl.disable(&req.into_inner().id);
        Ok(Response::new(DisableBotResponse { ok }))
    }

    async fn health(
        &self,
        _req: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            ok: true,
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }))
    }
}
