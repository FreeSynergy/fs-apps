pub mod app;
pub mod themes_view;
pub mod colors_view;
pub mod cursor_view;
pub mod chrome_view;

pub use app::ThemeManagerApp;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-theme-app (`theme.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-theme-app" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}
