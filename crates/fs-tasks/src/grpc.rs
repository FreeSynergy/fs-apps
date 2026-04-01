// grpc.rs — gRPC service implementation for fs-tasks.

use tonic::{Request, Response, Status};

use crate::controller::TaskController;
use crate::model::TaskPipeline;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("tasks");
}

pub use proto::tasks_service_server::{TasksService, TasksServiceServer};
pub use proto::{
    CreateTaskRequest, CreateTaskResponse, DeleteTaskRequest, DeleteTaskResponse, HealthRequest,
    HealthResponse, ListTasksRequest, ListTasksResponse, TaskProto, ToggleTaskRequest,
    ToggleTaskResponse,
};

fn to_proto(task: &TaskPipeline) -> TaskProto {
    TaskProto {
        id: task.id.clone(),
        name: task.name.clone(),
        enabled: task.enabled,
        trigger_label: task.trigger.label(),
        source_service: task.source.service.clone(),
        target_service: task.target.service.clone(),
    }
}

/// gRPC service backed by a shared [`TaskController`].
pub struct GrpcTasksApp {
    ctrl: TaskController,
}

impl GrpcTasksApp {
    #[must_use]
    pub fn new(ctrl: TaskController) -> Self {
        Self { ctrl }
    }
}

#[tonic::async_trait]
impl TasksService for GrpcTasksApp {
    async fn list_tasks(
        &self,
        _req: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let tasks = self.ctrl.list().iter().map(to_proto).collect();
        Ok(Response::new(ListTasksResponse { tasks }))
    }

    async fn create_task(
        &self,
        req: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let task = self.ctrl.create(req.into_inner().name);
        Ok(Response::new(CreateTaskResponse {
            task: Some(to_proto(&task)),
        }))
    }

    async fn delete_task(
        &self,
        req: Request<DeleteTaskRequest>,
    ) -> Result<Response<DeleteTaskResponse>, Status> {
        let ok = self.ctrl.delete(&req.into_inner().id);
        Ok(Response::new(DeleteTaskResponse { ok }))
    }

    async fn toggle_task(
        &self,
        req: Request<ToggleTaskRequest>,
    ) -> Result<Response<ToggleTaskResponse>, Status> {
        match self.ctrl.toggle(&req.into_inner().id) {
            Some(enabled) => Ok(Response::new(ToggleTaskResponse { ok: true, enabled })),
            None => Ok(Response::new(ToggleTaskResponse {
                ok: false,
                enabled: false,
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
