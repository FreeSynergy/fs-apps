pub mod app;
pub mod browser;
pub mod missing_icon;
pub mod node_package;
pub mod installed_list;
pub mod package_card;
pub mod install_wizard;
pub mod package_detail;
pub mod state;
pub mod store_settings;

pub use app::StoreApp;
pub use install_wizard::{do_install, InstallPopup, InstallResult};
pub use state::{notify_install_changed, INSTALL_COUNTER};

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-store-app (`store.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &str { "fs-store-app" }
    fn snippets(&self) -> &[(&str, &str)] { I18N_SNIPPETS }
}
