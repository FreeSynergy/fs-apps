#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::missing_panics_doc)]

pub mod bot_strategy;
pub mod cli;
pub mod controller;
pub mod grpc;
pub mod model;
pub mod rest;
pub mod view;

pub use controller::BotController;
pub use model::{MessagingBot, MessagingBotsConfig};
pub use view::BotsView;
