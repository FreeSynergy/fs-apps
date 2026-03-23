/// Theme Manager panel — shows active theme, lists available, allows switching.
use dioxus::prelude::*;
use fs_i18n;
use fs_manager_theme::ThemeManager;

use crate::picker_panel::{PickerItem, PickerPanel};

#[component]
pub fn ThemeManagerPanel() -> Element {
    let mgr   = ThemeManager::with_noop();
    let items: Vec<PickerItem> = mgr.available()
        .into_iter()
        .map(|t| {
            let icon  = if t.is_dark { "🌙" } else { "☀" };
            let badge = if t.is_dark { "Dark" } else { "Light" };
            PickerItem::new(t.id, t.display_name)
                .with_icon(icon)
                .with_badge(badge)
        })
        .collect();
    let active_id = mgr.active().id;

    rsx! {
        PickerPanel {
            title: fs_i18n::t("managers.theme.title").to_string(),
            description: fs_i18n::t("managers.theme.description").to_string(),
            items,
            active_id,
            on_apply: move |id: String| {
                let _ = ThemeManager::with_noop().set_active(&id);
            },
        }
    }
}
