// grpc.rs — gRPC service implementation for fs-builder.

use tonic::{Request, Response, Status};

use crate::controller::BuilderController;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("builder_app");
}

pub use proto::builder_service_server::{BuilderService, BuilderServiceServer};
pub use proto::{HealthRequest, HealthResponse, StatusRequest, StatusResponse};

/// gRPC service backed by a shared `BuilderController`.
pub struct GrpcBuilderApp {
    ctrl: BuilderController,
}

impl GrpcBuilderApp {
    #[must_use]
    pub fn new(ctrl: BuilderController) -> Self {
        Self { ctrl }
    }
}

#[tonic::async_trait]
impl BuilderService for GrpcBuilderApp {
    async fn status(
        &self,
        _req: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let pipelines = self.ctrl.list();
        let status = if pipelines.is_empty() {
            "idle".to_string()
        } else {
            format!("{} pipeline(s) active", pipelines.len())
        };
        Ok(Response::new(StatusResponse { status }))
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
