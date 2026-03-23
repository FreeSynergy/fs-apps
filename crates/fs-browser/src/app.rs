// Browser app — URL bar + iframe + tabs + bookmarks + history.
//
// K1: Browser-View (URL bar, iframe rendering, tabs)
// K2: Download handler → S3 /shared/downloads/ (intercepted via URL pattern)
// K3: Service integration — accepts a `service_url` context set by Conductor
// K4: Bookmarks + History (in-memory, to be persisted via fs-db in production)

use dioxus::prelude::*;
use fs_components::FS_SIDEBAR_CSS;
use fs_i18n;

use crate::bookmarks::{add_bookmark, record_visit, remove_bookmark};
use crate::model::{Bookmark, BrowserTab, DownloadEntry, HistoryEntry};
use crate::search_engine::BrowserConfig;

// ── Browser sections ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum BrowserPanel {
    Browse,
    Bookmarks,
    History,
    Downloads,
}

// ── Context: external apps can request opening a URL ─────────────────────────

/// Set this context from outside (e.g. Conductor) to open a URL in the browser.
pub type BrowserUrlRequest = Signal<Option<String>>;

// ── BrowserApp ────────────────────────────────────────────────────────────────

/// Root browser component.
#[component]
pub fn BrowserApp() -> Element {
    // Tabs state
    let mut tabs: Signal<Vec<BrowserTab>> = use_signal(|| vec![BrowserTab::new(1)]);
    let mut active_tab: Signal<u32>       = use_signal(|| 1);
    let mut next_tab_id: Signal<u32>      = use_signal(|| 2);

    // Address bar input (reflects active tab's url while typing)
    let mut address_input: Signal<String> = use_signal(String::new);

    // Side panel
    let mut panel: Signal<BrowserPanel> = use_signal(|| BrowserPanel::Browse);

    // K4: Bookmarks + History (in-memory)
    let mut bookmarks: Signal<Vec<Bookmark>>      = use_signal(Vec::new);
    let mut history:   Signal<Vec<HistoryEntry>>  = use_signal(Vec::new);
    let downloads: Signal<Vec<DownloadEntry>> = use_signal(Vec::new);

    // Browser config (search engine)
    let browser_config: Signal<BrowserConfig> = use_signal(BrowserConfig::load);

    // Status message (bookmark added/removed feedback)
    let mut status_msg: Signal<Option<String>> = use_signal(|| None);

    // K3: Accept service URL requests from Conductor via context
    // try_consume_context returns None if no BrowserUrlRequest was provided upstream.
    let url_request: Option<BrowserUrlRequest> = try_consume_context::<BrowserUrlRequest>();
    use_effect(move || {
        if let Some(mut req) = url_request {
            let maybe_url = req.read().clone();
            if let Some(url) = maybe_url {
                navigate_to(&mut tabs, *active_tab.read(), &url, &mut history, &mut address_input);
                *req.write() = None;
            }
        }
    });

    // Current active tab URL for iframe src
    let current_url = tabs.read()
        .iter()
        .find(|t| t.id == *active_tab.read())
        .map(|t| t.url.clone())
        .unwrap_or_default();

    let current_url_for_reload = current_url.clone();
    let show_panel = *panel.read() != BrowserPanel::Browse;

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        style { "{BROWSER_CSS}" }

        div {
            class: "fs-browser",

            // ── Title bar ────────────────────────────────────────────────────
            div {
                class: "fs-browser__titlebar",
                h2 {
                    style: "margin: 0; font-size: 15px; font-weight: 600; color: var(--fs-color-text-primary);",
                    {fs_i18n::t("browser.title")}
                }
            }

            // ── Toolbar: nav + address bar + panel toggles ────────────────
            div {
                class: "fs-browser__toolbar",

                // Nav buttons
                button {
                    class: "fs-browser__nav-btn",
                    title: fs_i18n::t("browser.go_back").to_string(),
                    onclick: move |_| {
                        // Back navigation — in an iframe context use JS eval
                        // For now: no-op (history management per-tab not trivial in iframe)
                    },
                    "‹"
                }
                button {
                    class: "fs-browser__nav-btn",
                    title: fs_i18n::t("browser.go_forward").to_string(),
                    onclick: move |_| {},
                    "›"
                }
                button {
                    class: "fs-browser__nav-btn",
                    title: fs_i18n::t("browser.reload").to_string(),
                    onclick: move |_| {
                        let url = normalize_url(&current_url_for_reload, &browser_config.read());
                        navigate_to(&mut tabs, *active_tab.read(), &url, &mut history, &mut address_input);
                    },
                    "↺"
                }

                // Address bar
                input {
                    class: "fs-browser__address",
                    r#type: "text",
                    placeholder: fs_i18n::t("browser.url_placeholder").to_string(),
                    value: "{address_input}",
                    oninput: move |e| address_input.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let url = normalize_url(&address_input.read(), &browser_config.read());
                            navigate_to(&mut tabs, *active_tab.read(), &url, &mut history, &mut address_input);
                        }
                    },
                }

                // Bookmark toggle
                button {
                    class: "fs-browser__nav-btn",
                    title: fs_i18n::t("browser.bookmarks.add").to_string(),
                    onclick: move |_| {
                        let url = current_url.clone();
                        if !url.is_empty() {
                            if let Some(bm) = add_bookmark(&url, &url) {
                                bookmarks.write().push(bm);
                                status_msg.set(Some(fs_i18n::t("browser.bookmarks.added").to_string()));
                            }
                        }
                    },
                    "☆"
                }

                // Panel toggles
                button {
                    class: if *panel.read() == BrowserPanel::Bookmarks { "fs-browser__nav-btn fs-browser__nav-btn--active" } else { "fs-browser__nav-btn" },
                    title: fs_i18n::t("browser.bookmarks.label").to_string(),
                    onclick: move |_| {
                        let p = if *panel.read() == BrowserPanel::Bookmarks { BrowserPanel::Browse } else { BrowserPanel::Bookmarks };
                        panel.set(p);
                    },
                    "🔖"
                }
                button {
                    class: if *panel.read() == BrowserPanel::History { "fs-browser__nav-btn fs-browser__nav-btn--active" } else { "fs-browser__nav-btn" },
                    title: fs_i18n::t("browser.history.label").to_string(),
                    onclick: move |_| {
                        let p = if *panel.read() == BrowserPanel::History { BrowserPanel::Browse } else { BrowserPanel::History };
                        panel.set(p);
                    },
                    "⏱"
                }
                button {
                    class: if *panel.read() == BrowserPanel::Downloads { "fs-browser__nav-btn fs-browser__nav-btn--active" } else { "fs-browser__nav-btn" },
                    title: fs_i18n::t("browser.downloads.label").to_string(),
                    onclick: move |_| {
                        let p = if *panel.read() == BrowserPanel::Downloads { BrowserPanel::Browse } else { BrowserPanel::Downloads };
                        panel.set(p);
                    },
                    "⬇"
                }
            }

            // ── Tab bar ───────────────────────────────────────────────────
            div {
                class: "fs-browser__tabbar",

                for tab in tabs.read().clone().iter() {
                    div {
                        key: "{tab.id}",
                        class: if tab.id == *active_tab.read() { "fs-browser__tab fs-browser__tab--active" } else { "fs-browser__tab" },
                        onclick: {
                            let tab_id = tab.id;
                            let url = tab.url.clone();
                            move |_| {
                                active_tab.set(tab_id);
                                address_input.set(url.clone());
                            }
                        },
                        span {
                            class: "fs-browser__tab-title",
                            "{tab.title}"
                        }
                        button {
                            class: "fs-browser__tab-close",
                            title: fs_i18n::t("browser.close_tab").to_string(),
                            onclick: {
                                let tab_id = tab.id;
                                move |e: MouseEvent| {
                                    e.stop_propagation();
                                    close_tab(&mut tabs, &mut active_tab, tab_id);
                                }
                            },
                            "✕"
                        }
                    }
                }

                // New tab button
                button {
                    class: "fs-browser__new-tab",
                    title: fs_i18n::t("browser.new_tab").to_string(),
                    onclick: move |_| {
                        let id = *next_tab_id.read();
                        next_tab_id.set(id + 1);
                        tabs.write().push(BrowserTab::new(id));
                        active_tab.set(id);
                        address_input.set(String::new());
                    },
                    "+"
                }
            }

            // ── Status message (transient) ────────────────────────────────
            if let Some(msg) = status_msg.read().clone() {
                div {
                    class: "fs-browser__status",
                    "{msg}"
                }
            }

            // ── Content area: iframe + optional side panel ─────────────────
            div {
                class: "fs-browser__content",

                // Main iframe viewport
                div {
                    class: "fs-browser__viewport",
                    style: if show_panel { "min-width: 0;" } else { "" },

                    if current_url.is_empty() {
                        // Empty tab — show a welcome page
                        div {
                            class: "fs-browser__newtab",
                            span { style: "font-size: 48px;", "🌐" }
                            p {
                                style: "color: var(--fs-color-text-muted); margin-top: 12px;",
                                {fs_i18n::t("browser.url_placeholder")}
                            }
                        }
                    } else {
                        // K1: WebView via iframe (Dioxus desktop runs in Wry/WebView).
                        // flex: 1 fills the flex-column parent; min-height: 0 prevents
                        // the flex minimum-size heuristic from collapsing the iframe to 0.
                        iframe {
                            key: "{current_url}",
                            src: "{current_url}",
                            style: "flex: 1; min-height: 0; width: 100%; border: none; display: block;",
                        }
                    }
                }

                // Side panel (bookmarks / history / downloads)
                if show_panel {
                    div {
                        class: "fs-browser__panel",

                        match *panel.read() {
                            BrowserPanel::Bookmarks => rsx! {
                                BookmarksPanel {
                                    bookmarks: bookmarks.read().clone(),
                                    on_open: move |url: String| {
                                        navigate_to(&mut tabs, *active_tab.read(), &url, &mut history, &mut address_input);
                                        panel.set(BrowserPanel::Browse);
                                    },
                                    on_remove: move |id: i64| {
                                        remove_bookmark(&mut bookmarks.write(), id);
                                    },
                                }
                            },
                            BrowserPanel::History => rsx! {
                                HistoryPanel {
                                    history: history.read().clone(),
                                    on_open: move |url: String| {
                                        navigate_to(&mut tabs, *active_tab.read(), &url, &mut history, &mut address_input);
                                        panel.set(BrowserPanel::Browse);
                                    },
                                    on_clear: move |_| {
                                        crate::history::clear(&mut history.write());
                                    },
                                }
                            },
                            BrowserPanel::Downloads => rsx! {
                                DownloadsPanel {
                                    downloads: downloads.read().clone(),
                                }
                            },
                            BrowserPanel::Browse => rsx! {},
                        }
                    }
                }
            }
        }
    }
}

// ── BookmarksPanel ────────────────────────────────────────────────────────────

#[component]
fn BookmarksPanel(
    bookmarks: Vec<Bookmark>,
    on_open: EventHandler<String>,
    on_remove: EventHandler<i64>,
) -> Element {
    rsx! {
        div { class: "fs-browser__panel-inner",
            h3 { style: "margin: 0 0 12px; font-size: 14px;", {fs_i18n::t("browser.bookmarks.label")} }
            if bookmarks.is_empty() {
                p { style: "color: var(--fs-color-text-muted); font-size: 13px;",
                    {fs_i18n::t("browser.bookmarks.empty")}
                }
            }
            for bm in bookmarks.iter() {
                div { key: "{bm.id}",
                    class: "fs-browser__panel-row",
                    div {
                        class: "fs-browser__panel-row-main",
                        onclick: {
                            let url = bm.url.clone();
                            move |_| on_open.call(url.clone())
                        },
                        span {
                            style: "font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{bm.title}"
                        }
                    }
                    button {
                        class: "fs-browser__panel-row-action",
                        onclick: {
                            let id = bm.id;
                            move |_| on_remove.call(id)
                        },
                        title: fs_i18n::t("browser.bookmarks.remove").to_string(),
                        "✕"
                    }
                }
            }
        }
    }
}

// ── HistoryPanel ──────────────────────────────────────────────────────────────

#[component]
fn HistoryPanel(
    history: Vec<HistoryEntry>,
    on_open: EventHandler<String>,
    on_clear: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "fs-browser__panel-inner",
            div { style: "display: flex; align-items: center; gap: 8px; margin-bottom: 12px;",
                h3 { style: "margin: 0; flex: 1; font-size: 14px;", {fs_i18n::t("browser.history.label")} }
                button {
                    class: "fs-browser__panel-row-action",
                    onclick: move |_| on_clear.call(()),
                    {fs_i18n::t("browser.history.clear")}
                }
            }
            if history.is_empty() {
                p { style: "color: var(--fs-color-text-muted); font-size: 13px;",
                    {fs_i18n::t("browser.history.empty")}
                }
            }
            for entry in crate::history::recent(&history, 100).into_iter() {
                div { key: "{entry.id}",
                    class: "fs-browser__panel-row",
                    div {
                        class: "fs-browser__panel-row-main",
                        onclick: {
                            let url = entry.url.clone();
                            move |_| on_open.call(url.clone())
                        },
                        span { style: "font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{entry.title}"
                        }
                        span { style: "font-size: 11px; color: var(--fs-color-text-muted);",
                            " — {entry.visited_at}"
                        }
                    }
                }
            }
        }
    }
}

// ── DownloadsPanel ────────────────────────────────────────────────────────────

#[component]
fn DownloadsPanel(downloads: Vec<DownloadEntry>) -> Element {
    rsx! {
        div { class: "fs-browser__panel-inner",
            h3 { style: "margin: 0 0 12px; font-size: 14px;", {fs_i18n::t("browser.downloads.label")} }
            if downloads.is_empty() {
                p { style: "color: var(--fs-color-text-muted); font-size: 13px;",
                    {fs_i18n::t("browser.downloads.empty")}
                }
            }
            for dl in downloads.iter() {
                div { key: "{dl.id}",
                    class: "fs-browser__panel-row",
                    div { class: "fs-browser__panel-row-main",
                        span { style: "font-size: 13px;", "{dl.filename}" }
                        span { style: "font-size: 11px; color: var(--fs-color-text-muted);",
                            " → {dl.s3_path}"
                        }
                    }
                    span {
                        style: "font-size: 12px; color: var(--fs-color-text-muted); white-space: nowrap;",
                        "{dl.status.label()}"
                    }
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Navigate the active tab to `url`.
fn navigate_to(
    tabs:    &mut Signal<Vec<BrowserTab>>,
    tab_id:  u32,
    url:     &str,
    history: &mut Signal<Vec<HistoryEntry>>,
    address: &mut Signal<String>,
) {
    // navigate_to is called with already-normalized URLs; keep as-is.
    let url = url.to_string();
    address.set(url.clone());
    history.write().push(record_visit(&url, &url));

    if let Some(tab) = tabs.write().iter_mut().find(|t| t.id == tab_id) {
        tab.navigate(url);
    }
}

/// Close a tab; switch to the adjacent one if the closed tab was active.
fn close_tab(tabs: &mut Signal<Vec<BrowserTab>>, active: &mut Signal<u32>, id: u32) {
    let len = tabs.read().len();
    if len <= 1 {
        // Always keep at least one tab; reset it instead
        if let Some(t) = tabs.write().first_mut() {
            t.reset();
        }
        return;
    }

    let idx = tabs.read().iter().position(|t| t.id == id).unwrap_or(0);
    tabs.write().remove(idx);

    if *active.read() == id {
        let new_idx = idx.saturating_sub(1).min(tabs.read().len().saturating_sub(1));
        if let Some(t) = tabs.read().get(new_idx) {
            active.set(t.id);
        }
    }
}

/// Normalize an address bar input to a URL.
///
/// - Already a URL (has scheme) → returned as-is
/// - Looks like a domain (`foo.com`) → prepend `https://`
/// - Anything else → treat as search query, build URL via configured engine
fn normalize_url(input: &str, config: &BrowserConfig) -> String {
    let t = input.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.starts_with("http://") || t.starts_with("https://") || t.starts_with("file://") {
        t.to_string()
    } else if t.contains('.') && !t.contains(' ') {
        format!("https://{t}")
    } else {
        config.build_search_url(t)
    }
}

// ── CSS ───────────────────────────────────────────────────────────────────────

const BROWSER_CSS: &str = r#"
.fs-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--fs-color-bg-base);
    font-family: var(--fs-font-family, system-ui, sans-serif);
}

.fs-browser__titlebar {
    padding: 8px 16px;
    border-bottom: 1px solid var(--fs-color-border-default);
    flex-shrink: 0;
    background: var(--fs-color-bg-surface);
}

.fs-browser__toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--fs-color-border-default);
    background: var(--fs-color-bg-surface);
    flex-shrink: 0;
}

.fs-browser__nav-btn {
    background: transparent;
    border: none;
    color: var(--fs-color-text-muted);
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background 100ms;
    white-space: nowrap;
}
.fs-browser__nav-btn:hover {
    background: var(--fs-color-bg-elevated);
}
.fs-browser__nav-btn--active {
    color: var(--fs-color-primary, #06b6d4);
}

.fs-browser__address {
    flex: 1;
    background: var(--fs-color-bg-base);
    border: 1px solid var(--fs-color-border-default);
    border-radius: 6px;
    color: var(--fs-color-text-primary);
    font-size: 13px;
    padding: 5px 10px;
    outline: none;
    transition: border-color 150ms;
}
.fs-browser__address:focus {
    border-color: var(--fs-color-primary, #06b6d4);
}

.fs-browser__tabbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 8px 0;
    background: var(--fs-color-bg-surface);
    border-bottom: 1px solid var(--fs-color-border-default);
    overflow-x: auto;
    flex-shrink: 0;
}

.fs-browser__tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: 6px 6px 0 0;
    cursor: pointer;
    font-size: 12px;
    max-width: 160px;
    color: var(--fs-color-text-muted);
    background: transparent;
    transition: background 100ms;
    white-space: nowrap;
}
.fs-browser__tab:hover {
    background: var(--fs-color-bg-elevated);
}
.fs-browser__tab--active {
    background: var(--fs-color-bg-base);
    color: var(--fs-color-text-primary);
    border-bottom: 2px solid var(--fs-color-primary, #06b6d4);
}

.fs-browser__tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 110px;
}

.fs-browser__tab-close {
    background: transparent;
    border: none;
    color: var(--fs-color-text-muted);
    cursor: pointer;
    font-size: 10px;
    padding: 0 2px;
    border-radius: 3px;
    flex-shrink: 0;
}
.fs-browser__tab-close:hover {
    background: rgba(239,68,68,0.2);
    color: #ef4444;
}

.fs-browser__new-tab {
    background: transparent;
    border: none;
    color: var(--fs-color-text-muted);
    cursor: pointer;
    font-size: 18px;
    padding: 2px 8px;
    border-radius: 4px;
    flex-shrink: 0;
}
.fs-browser__new-tab:hover {
    background: var(--fs-color-bg-elevated);
}

.fs-browser__status {
    padding: 4px 16px;
    font-size: 11px;
    color: var(--fs-color-primary, #06b6d4);
    background: var(--fs-color-bg-surface);
    flex-shrink: 0;
}

.fs-browser__content {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
}

.fs-browser__viewport {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
}

.fs-browser__newtab {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--fs-color-text-muted);
}

.fs-browser__panel {
    width: 280px;
    flex-shrink: 0;
    border-left: 1px solid var(--fs-color-border-default);
    background: var(--fs-color-bg-surface);
    overflow-y: auto;
}

.fs-browser__panel-inner {
    padding: 16px;
}

.fs-browser__panel-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid var(--fs-color-border-subtle, rgba(255,255,255,0.05));
}

.fs-browser__panel-row-main {
    flex: 1;
    min-width: 0;
    cursor: pointer;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.fs-browser__panel-row-main:hover span:first-child {
    color: var(--fs-color-primary, #06b6d4);
}

.fs-browser__panel-row-action {
    background: transparent;
    border: none;
    color: var(--fs-color-text-muted);
    cursor: pointer;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    white-space: nowrap;
}
.fs-browser__panel-row-action:hover {
    background: var(--fs-color-bg-elevated);
}
"#;
