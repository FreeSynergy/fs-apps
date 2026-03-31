use dioxus::prelude::*;

const I18N_SNIPPETS: &[(&str, &str)] = &[
    ("en", include_str!("../assets/i18n/en.toml")),
    ("de", include_str!("../assets/i18n/de.toml")),
];

/// i18n plugin for fs-builder (`builder.*` keys).
pub struct I18nPlugin;

impl fs_i18n::SnippetPlugin for I18nPlugin {
    fn name(&self) -> &'static str {
        "fs-builder"
    }
    fn snippets(&self) -> &[(&str, &str)] {
        I18N_SNIPPETS
    }
}

/// Builder app — package creation and publishing UI.
#[component]
pub fn BuilderApp() -> Element {
    rsx! {
        div { class: "builder-placeholder",
            h2 { "Builder" }
            p { "Package builder coming soon." }
        }
    }
}
