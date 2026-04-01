//! `fs-builder` — FreeSynergy package builder.
//!
//! Pipeline Pattern:
//! - [`BuildPipeline`] — Analyse → Validate → Build → Publish
//! - [`BuildStep`] — Chain of Responsibility trait
//! - [`BuilderView`] — `FsView` impl (in `view.rs`, only file importing fs-render)

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

pub use controller::BuilderController;
pub use model::{BuildPipeline, BuildStep, BuildStepKind, BuildStepStatus};
pub use view::BuilderView;
