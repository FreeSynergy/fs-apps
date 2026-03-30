#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
pub mod app;
pub mod model;
mod pipeline_editor;
mod templates;

pub use app::TasksApp;
