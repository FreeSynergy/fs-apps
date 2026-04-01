// grpc.rs — gRPC service implementation for fs-lenses.

use tonic::{Request, Response, Status};

use crate::controller::LensController;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("lens");
}

pub use proto::lens_service_server::{LensService, LensServiceServer};
pub use proto::{
    CreateLensRequest, CreateLensResponse, DeleteLensRequest, DeleteLensResponse, HealthRequest,
    HealthResponse, LensProto, ListLensesRequest, ListLensesResponse, QueryLensRequest,
    QueryLensResponse,
};

fn to_proto(lens: &crate::model::Lens) -> LensProto {
    LensProto {
        id: lens.id,
        name: lens.name.clone(),
        query: lens.query.clone(),
    }
}

/// gRPC service backed by a shared `LensController`.
pub struct GrpcLensApp {
    ctrl: LensController,
}

impl GrpcLensApp {
    #[must_use]
    pub fn new(ctrl: LensController) -> Self {
        Self { ctrl }
    }
}

#[tonic::async_trait]
impl LensService for GrpcLensApp {
    async fn list_lenses(
        &self,
        _req: Request<ListLensesRequest>,
    ) -> Result<Response<ListLensesResponse>, Status> {
        let lenses = self.ctrl.list().iter().map(to_proto).collect();
        Ok(Response::new(ListLensesResponse { lenses }))
    }

    async fn create_lens(
        &self,
        req: Request<CreateLensRequest>,
    ) -> Result<Response<CreateLensResponse>, Status> {
        let inner = req.into_inner();
        let lens = self.ctrl.create(inner.name, inner.query);
        Ok(Response::new(CreateLensResponse {
            lens: Some(to_proto(&lens)),
        }))
    }

    async fn delete_lens(
        &self,
        req: Request<DeleteLensRequest>,
    ) -> Result<Response<DeleteLensResponse>, Status> {
        let id = req.into_inner().id;
        let ok = self.ctrl.delete(id);
        Ok(Response::new(DeleteLensResponse { ok }))
    }

    async fn query_lens(
        &self,
        req: Request<QueryLensRequest>,
    ) -> Result<Response<QueryLensResponse>, Status> {
        let inner = req.into_inner();
        let results = self.ctrl.query(inner.lens_id, &inner.query);
        Ok(Response::new(QueryLensResponse { results }))
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
