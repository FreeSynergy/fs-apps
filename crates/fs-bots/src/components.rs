/// Shared UI components and the ListRowContent trait for display-only rows.
use dioxus::prelude::*;

// ── EmptyState ─────────────────────────────────────────────────────────────────

/// Centered empty-state message box — replaces repeated inline patterns.
#[component]
pub fn EmptyState(message: String) -> Element {
    rsx! {
        div {
            style: "background: var(--fs-color-bg-overlay); \
                    border-radius: var(--fs-radius-md); \
                    padding: 20px; text-align: center; \
                    color: var(--fs-color-text-muted); font-size: 13px;",
            "{message}"
        }
    }
}

// ── StatusDot ──────────────────────────────────────────────────────────────────

/// Connection status indicator: coloured dot + label.
#[component]
pub fn StatusDot(connected: bool) -> Element {
    let color = if connected { "#22c55e" } else { "#ef4444" };
    let label = if connected { "Connected" } else { "Disconnected" };
    rsx! {
        div { style: "display: flex; align-items: center; gap: 6px; flex-shrink: 0;",
            div { style: "width: 8px; height: 8px; border-radius: 50%; background: {color};" }
            span { style: "font-size: 12px; color: {color};", "{label}" }
        }
    }
}

// ── ListRowContent ─────────────────────────────────────────────────────────────

/// Trait for display-only list rows.
/// Implementors provide icon, body, and optional trailing content.
/// Interactive rows (with callbacks) should remain as custom components.
pub trait ListRowContent: Clone + PartialEq + 'static {
    fn row_icon(&self) -> Element;
    fn row_body(&self) -> Element;
    fn row_trailing(&self) -> Element;
}

// ── ListRow ────────────────────────────────────────────────────────────────────

/// Generic display-only list row — renders any `ListRowContent` implementor.
#[component]
pub fn ListRow<T: ListRowContent>(item: T) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; font-size: 12px; \
                    color: var(--fs-color-text-muted); padding: 5px 0; \
                    border-bottom: 1px solid var(--fs-color-border-default);",
            { item.row_icon() }
            { item.row_body() }
            { item.row_trailing() }
        }
    }
}
