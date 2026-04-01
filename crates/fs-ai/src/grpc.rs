// grpc.rs — gRPC service implementation for fs-ai.

use tonic::{Request, Response, Status};

use crate::controller::AiController;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("ai");
}

pub use proto::ai_service_server::{AiService, AiServiceServer};
pub use proto::{
    GetStatusRequest, GetStatusResponse, HealthRequest, HealthResponse, ListModelsRequest,
    ListModelsResponse, ModelProto, StartEngineRequest, StartEngineResponse, StopEngineRequest,
    StopEngineResponse,
};

/// gRPC service backed by a shared [`AiController`].
pub struct GrpcAiApp {
    ctrl: AiController,
}

impl GrpcAiApp {
    #[must_use]
    pub fn new(ctrl: AiController) -> Self {
        Self { ctrl }
    }
}

#[tonic::async_trait]
impl AiService for GrpcAiApp {
    async fn list_models(
        &self,
        _req: Request<ListModelsRequest>,
    ) -> Result<Response<ListModelsResponse>, Status> {
        let models = self
            .ctrl
            .list_models()
            .into_iter()
            .map(|m| ModelProto {
                id: m.id,
                name: m.name,
            })
            .collect();
        Ok(Response::new(ListModelsResponse { models }))
    }

    async fn get_status(
        &self,
        _req: Request<GetStatusRequest>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        let snap = self.ctrl.snapshot();
        Ok(Response::new(GetStatusResponse {
            running: snap.running,
            port: snap.port.map_or(0, u32::from),
            api_url: snap.api_url().unwrap_or_default(),
        }))
    }

    async fn start_engine(
        &self,
        req: Request<StartEngineRequest>,
    ) -> Result<Response<StartEngineResponse>, Status> {
        match self.ctrl.start(&req.into_inner().model_id) {
            Ok(_) => Ok(Response::new(StartEngineResponse {
                ok: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(StartEngineResponse {
                ok: false,
                error: e,
            })),
        }
    }

    async fn stop_engine(
        &self,
        _req: Request<StopEngineRequest>,
    ) -> Result<Response<StopEngineResponse>, Status> {
        let ok = self.ctrl.stop().is_ok();
        Ok(Response::new(StopEngineResponse { ok }))
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
