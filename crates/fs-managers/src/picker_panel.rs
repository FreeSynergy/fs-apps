/// Generic picker panel — reusable "select one item from a list" UI.
///
/// Used by `ThemeManagerPanel` and `LanguageManagerPanel` to avoid
/// duplicating the list + Apply button pattern.
///
/// # Usage
///
/// ```rust,ignore
/// PickerPanel {
///     title: fs_i18n::t("managers.theme.title").to_string(),
///     description: fs_i18n::t("managers.theme.description").to_string(),
///     items: themes.iter().map(PickerItem::from_theme).collect(),
///     active_id: active.id.clone(),
///     on_apply: move |id| { ThemeManager::new().set_active(&id).ok(); },
/// }
/// ```
use dioxus::prelude::*;

// ── PickerItem ─────────────────────────────────────────────────────────────────

/// One selectable item in a [`PickerPanel`].
#[derive(Clone, PartialEq)]
pub struct PickerItem {
    /// Stable identifier passed to `on_apply` when this item is selected.
    pub id: String,
    /// Primary label shown in the list row.
    pub label: String,
    /// Optional emoji / text icon shown on the left.
    pub icon: Option<String>,
    /// Optional inline HTML (e.g. SVG flag) shown between icon and label.
    pub icon_html: Option<String>,
    /// Optional right-aligned badge (e.g. "Dark", "de-DE").
    pub badge: Option<String>,
}

impl PickerItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            icon_html: None,
            badge: None,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_icon_html(mut self, html: impl Into<String>) -> Self {
        self.icon_html = Some(html.into());
        self
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

// ── PickerPanel ───────────────────────────────────────────────────────────────

/// A panel that lists selectable items and provides an Apply button.
///
/// Manages its own selection state (signals). The caller only needs to
/// supply the item list, the current active id, and an `on_apply` handler.
#[component]
pub fn PickerPanel(
    title: String,
    description: String,
    items: Vec<PickerItem>,
    active_id: String,
    on_apply: EventHandler<String>,
) -> Element {
    let mut selected = use_signal(|| active_id.clone());
    let mut saved    = use_signal(|| false);

    rsx! {
        div {
            style: "padding: 24px; max-width: 480px;",

            h3 { style: "margin-top: 0; color: var(--fs-text-primary);",
                "{title}"
            }
            p { style: "font-size: 13px; color: var(--fs-color-text-muted); margin-top: -8px;",
                "{description}"
            }

            div {
                style: "border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-md); overflow: hidden; margin-bottom: 20px;",

                for item in &items {
                    {
                        let is_active = item.id == *selected.read();
                        let item_id   = item.id.clone();
                        let bg = if is_active {
                            "background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15)); \
                             color: var(--fs-sidebar-active, #4d8bf5);"
                        } else {
                            "background: transparent; color: var(--fs-color-text-primary);"
                        };
                        let icon_html = item.icon_html.clone();
                        let icon      = item.icon.clone();
                        let label     = item.label.clone();
                        let badge     = item.badge.clone();
                        rsx! {
                            div {
                                key: "{item_id}",
                                style: "display: flex; align-items: center; gap: 12px; \
                                        padding: 11px 16px; cursor: pointer; \
                                        border-bottom: 1px solid var(--fs-color-border-default); \
                                        transition: background 100ms; {bg}",
                                onclick: move |_| {
                                    selected.set(item_id.clone());
                                    saved.set(false);
                                },

                                // Selection indicator
                                span { style: "font-size: 16px; flex-shrink: 0;",
                                    if is_active { "◉" } else { "○" }
                                }

                                // Emoji icon (optional)
                                if let Some(ref ic) = icon {
                                    span { style: "font-size: 18px; flex-shrink: 0;",
                                        "{ic}"
                                    }
                                }

                                // Inline HTML icon (e.g. SVG flag — optional)
                                if let Some(ref html) = icon_html {
                                    if !html.is_empty() {
                                        span {
                                            style: "flex-shrink: 0; width: 24px; height: 14px; \
                                                    display: inline-flex; align-items: center; \
                                                    border-radius: 2px; overflow: hidden;",
                                            dangerous_inner_html: "{html}",
                                        }
                                    }
                                }

                                // Primary label
                                span { style: "font-size: 14px; font-weight: 500; flex: 1;",
                                    "{label}"
                                }

                                // Badge (optional)
                                if let Some(ref b) = badge {
                                    span {
                                        style: "font-size: 11px; padding: 2px 8px; \
                                                border-radius: 999px; opacity: 0.7; \
                                                background: var(--fs-color-bg-overlay);",
                                        "{b}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { style: "display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fs-color-primary, #06b6d4); \
                            color: white; border: none; border-radius: var(--fs-radius-md, 6px); \
                            cursor: pointer; font-size: 13px;",
                    onclick: move |_| {
                        let id = selected.read().clone();
                        on_apply.call(id);
                        saved.set(true);
                    },
                    {fs_i18n::t("actions.apply")}
                }
                if *saved.read() {
                    span { style: "font-size: 12px; color: var(--fs-color-text-muted);",
                        {fs_i18n::t("managers.saved")}
                    }
                }
            }
        }
    }
}
