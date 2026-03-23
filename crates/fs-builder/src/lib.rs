pub mod app;
pub mod bridge_builder;
pub mod container_builder;
pub mod i18n_editor;
pub mod ollama;
pub mod resource_browser;

pub use app::BuilderApp;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-builder (`builder.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-builder" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}
