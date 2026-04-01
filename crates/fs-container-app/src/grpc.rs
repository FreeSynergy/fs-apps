// grpc.rs — gRPC service implementation for fs-container-app.

use std::sync::Arc;

use fs_container::ContainerEngine;
use tonic::{Request, Response, Status};

use crate::controller::ContainerAppController;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("container_app");
}

pub use proto::container_app_service_server::{ContainerAppService, ContainerAppServiceServer};
pub use proto::{
    HealthRequest, HealthResponse, ListServicesRequest, ListServicesResponse, ServiceProto,
    StartServiceRequest, StartServiceResponse, StopServiceRequest, StopServiceResponse,
};

/// gRPC service backed by a shared [`ContainerAppController`].
pub struct GrpcContainerApp<E: ContainerEngine + 'static> {
    ctrl: Arc<ContainerAppController<E>>,
}

impl<E: ContainerEngine + 'static> GrpcContainerApp<E> {
    #[must_use]
    pub fn new(ctrl: ContainerAppController<E>) -> Self {
        Self {
            ctrl: Arc::new(ctrl),
        }
    }
}

#[tonic::async_trait]
impl<E: ContainerEngine + 'static> ContainerAppService for GrpcContainerApp<E> {
    async fn list_services(
        &self,
        _req: Request<ListServicesRequest>,
    ) -> Result<Response<ListServicesResponse>, Status> {
        self.ctrl.refresh().await;
        let services = self
            .ctrl
            .snapshot()
            .containers
            .into_iter()
            .map(|c| ServiceProto {
                name: c.name,
                state_label: c.state_label,
            })
            .collect();
        Ok(Response::new(ListServicesResponse { services }))
    }

    async fn start_service(
        &self,
        req: Request<StartServiceRequest>,
    ) -> Result<Response<StartServiceResponse>, Status> {
        let name = req.into_inner().name;
        match self.ctrl.start(&name).await {
            Ok(()) => Ok(Response::new(StartServiceResponse {
                ok: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(StartServiceResponse {
                ok: false,
                error: e.to_string(),
            })),
        }
    }

    async fn stop_service(
        &self,
        req: Request<StopServiceRequest>,
    ) -> Result<Response<StopServiceResponse>, Status> {
        let name = req.into_inner().name;
        match self.ctrl.stop(&name).await {
            Ok(()) => Ok(Response::new(StopServiceResponse {
                ok: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(StopServiceResponse {
                ok: false,
                error: e.to_string(),
            })),
        }
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
