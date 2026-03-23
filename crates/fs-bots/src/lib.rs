pub mod app;
pub mod bot_strategy;
pub mod components;
pub mod context;
pub mod model;
pub mod view_trait;

pub mod accounts_view;
pub mod broadcast_view;
pub mod gatekeeper_view;
pub mod groups_view;

pub use app::BotManagerApp;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-bots (`bots.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-bots" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}
