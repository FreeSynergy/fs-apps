pub mod app;
pub mod model;
pub mod query;

pub use app::LensesApp;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-lenses (`lenses.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-lenses" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}
