//! `fs-lenses` — aggregated cross-service data views for FreeSynergy.
//!
//! Implements the Strategy Pattern:
//! - [`LensController`] — CRUD + query dispatch (knows only traits)
//! - [`LensesView`] / [`LensDetailView`] — `FsView` impls (in `view.rs`)
//! - [`GrpcLensApp`] — gRPC service
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
pub mod query;
pub mod rest;
pub mod view;

pub use controller::LensController;
pub use model::Lens;
pub use view::{LensDetailView, LensesView};
