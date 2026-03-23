pub mod app;
pub mod build_view;
pub mod instance_config;
pub mod log_viewer;
pub mod service_detail;
pub mod service_list;

pub use app::Container;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-container-app (`container.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-container-app" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}
