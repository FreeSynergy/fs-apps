#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
pub mod app;
pub mod container_panel;
pub mod cursor_panel;
pub mod icons_panel;
pub mod language_panel;
pub mod manager_view;
pub mod package_type_display;
pub mod picker_panel;
pub mod theme_panel;
pub mod view_model;

pub use app::ManagersApp;
pub use container_panel::ContainerManagerPanel;
pub use cursor_panel::CursorManagerPanel;
pub use icons_panel::IconsManagerPanel;
pub use language_panel::LanguageManagerPanel;
pub use manager_view::ManagerView;
pub use package_type_display::PackageTypeDisplay;
pub use picker_panel::{PickerItem, PickerPanel};
pub use theme_panel::ThemeManagerPanel;
pub use view_model::PackageViewModel;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-managers (`managers.*` keys). Pass to [`fs_i18n::init_with_plugins`].
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &'static str {
        "fs-managers"
    }
    fn snippets(&self) -> &[(&str, &str)] {
        I18N_SNIPPETS
    }
}
