// grpc.rs — gRPC service implementation for fs-theme-app.

use tonic::{Request, Response, Status};

use crate::controller::ThemeController;

pub mod proto {
    #![allow(clippy::all, clippy::pedantic, warnings)]
    tonic::include_proto!("theme_app");
}

pub use proto::theme_app_service_server::{ThemeAppService, ThemeAppServiceServer};
pub use proto::{
    ActivateThemeRequest, ActivateThemeResponse, GetActiveRequest, GetActiveResponse,
    HealthRequest, HealthResponse, ListThemesRequest, ListThemesResponse, PreviewThemeRequest,
    PreviewThemeResponse, ThemeProto,
};

fn to_proto(info: &crate::controller::ThemeInfo) -> ThemeProto {
    ThemeProto {
        name: info.name.clone(),
        version: info.version.clone(),
    }
}

/// gRPC service backed by a shared `ThemeController`.
pub struct GrpcThemeApp {
    ctrl: ThemeController,
}

impl GrpcThemeApp {
    #[must_use]
    pub fn new(ctrl: ThemeController) -> Self {
        Self { ctrl }
    }
}

#[tonic::async_trait]
impl ThemeAppService for GrpcThemeApp {
    async fn list_themes(
        &self,
        _req: Request<ListThemesRequest>,
    ) -> Result<Response<ListThemesResponse>, Status> {
        let themes = self.ctrl.list().iter().map(to_proto).collect();
        Ok(Response::new(ListThemesResponse { themes }))
    }

    async fn get_active(
        &self,
        _req: Request<GetActiveRequest>,
    ) -> Result<Response<GetActiveResponse>, Status> {
        let active = self.ctrl.active();
        Ok(Response::new(GetActiveResponse {
            theme: Some(to_proto(&active)),
        }))
    }

    async fn activate_theme(
        &self,
        req: Request<ActivateThemeRequest>,
    ) -> Result<Response<ActivateThemeResponse>, Status> {
        let name = req.into_inner().name;
        match self.ctrl.activate(&name) {
            Ok(()) => Ok(Response::new(ActivateThemeResponse {
                ok: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ActivateThemeResponse {
                ok: false,
                error: e,
            })),
        }
    }

    async fn preview_theme(
        &self,
        req: Request<PreviewThemeRequest>,
    ) -> Result<Response<PreviewThemeResponse>, Status> {
        let name = req.into_inner().name;
        match self.ctrl.preview_css(&name) {
            Some(css) => Ok(Response::new(PreviewThemeResponse { css, found: true })),
            None => Ok(Response::new(PreviewThemeResponse {
                css: String::new(),
                found: false,
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
