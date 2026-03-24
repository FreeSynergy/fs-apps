pub mod app;
pub mod bookmarks;
pub mod history;
pub mod model;
pub mod search_engine;

pub use app::BrowserApp;
pub use search_engine::{BrowserConfig, SearchEngine, SearchEngineRegistry};

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-browser (`browser.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str {
        "fs-browser"
    }
    fn snippets(&self) -> &[(&str, &str)] {
        I18N_SNIPPETS
    }
}
