/// Bots Manager panel — placeholder until fs-manager-bot is implemented.
use dioxus::prelude::*;
use fs_i18n;

#[component]
pub fn BotsManagerPanel() -> Element {
    rsx! {
        div {
            style: "padding: 24px; max-width: 480px;",
            h3 { style: "margin-top: 0; color: var(--fs-text-primary);",
                {fs_i18n::t("managers.bots.title")}
            }
            div {
                style: "padding: 40px; text-align: center; \
                        color: var(--fs-color-text-muted); font-size: 13px; \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md);",
                span { style: "display: block; font-size: 36px; margin-bottom: 12px;", "🤖" }
                {fs_i18n::t("managers.bots.placeholder")}
            }
        }
    }
}
