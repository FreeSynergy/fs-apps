//! `fs-ai` — FreeSynergy AI assistant.
//!
//! Facade Pattern: [`AiController`] wraps `fs-manager-ai` (LLM engine management).
//!
//! - [`AiController`] — start/stop/status (knows only `AiEngine` trait)
//! - [`AiView`] — `FsView` impl (in `view.rs`, only file importing fs-render)
//! - [`GrpcAiApp`] — gRPC service
//! - REST router via [`rest::router`]
//! - CLI via [`cli::Cli`]

#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_for_each)]

pub mod cli;
pub mod controller;
pub mod grpc;
pub mod model;
pub mod rest;
pub mod view;

pub use controller::AiController;
pub use model::{AiModel, KnownModel};
pub use view::AiView;
