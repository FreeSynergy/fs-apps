/// Store — three-section layout: Server | Apps | Desktop.
///
/// Each section has its own sidebar. The pinned ⚙ Settings item is always
/// visible regardless of the active section. Switching sections triggers the
/// universal FsTabView slide+blur animation.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, FsTabDef, FsTabView,
                     FS_SIDEBAR_CSS, FS_TAB_VIEW_CSS};
use fs_store::StoreClient;

use crate::browser::PackageBrowser;
use crate::installed_list::InstalledList;
use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::PackageEntry;
use crate::package_detail::PackageDetail;
use crate::store_settings::StoreSettings;

// ── SVG icons ────────────────────────────────────────────────────────────────

const ICON_SERVER: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="4" rx="1"/><rect x="2" y="10" width="20" height="4" rx="1"/><rect x="2" y="17" width="20" height="4" rx="1"/><circle cx="6" cy="5" r="1" fill="currentColor"/><circle cx="6" cy="12" r="1" fill="currentColor"/><circle cx="6" cy="19" r="1" fill="currentColor"/></svg>"#;
const ICON_APPS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/></svg>"#;
const ICON_DESKTOP: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/></svg>"#;
const ICON_BUNDLE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>"#;
const ICON_BOT: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M12 11V3"/><circle cx="12" cy="3" r="1" fill="currentColor"/><line x1="8" y1="16" x2="8" y2="16" stroke-width="3"/><line x1="16" y1="16" x2="16" y2="16" stroke-width="3"/><path d="M9 20h6"/></svg>"#;
const ICON_THEME: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r="0.5" fill="currentColor"/><circle cx="17.5" cy="10.5" r="0.5" fill="currentColor"/><circle cx="8.5" cy="7.5" r="0.5" fill="currentColor"/><circle cx="6.5" cy="12.5" r="0.5" fill="currentColor"/><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10c.19 0 .37-.01.56-.02a1 1 0 0 0 .94-1V19a2 2 0 0 1 2-2h3a2 2 0 0 0 2-2v-1c0-5.52-4.48-10-10-10z"/></svg>"#;
const ICON_WIDGET: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>"#;
const ICON_ICONS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="4"/><path d="M6 20v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2"/></svg>"#;
const ICON_CURSOR: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 3l14 9-7 1-4 7z"/></svg>"#;
const ICON_LANG: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>"#;
const ICON_SETTINGS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"#;
const ICON_INSTALLED: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>"#;
const ICON_UPDATES: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/></svg>"#;
const ICON_BRIDGE: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>"#;

// ── StoreSection ─────────────────────────────────────────────────────────────

/// The three top-level sections of the Store.
///
/// Each section groups related package types and has its own sidebar navigation.
/// The active section switches via FsTabView with a slide+blur animation.
#[derive(Clone, PartialEq, Debug)]
pub enum StoreSection {
    /// Server-side: Container Apps, Bridges. Requires a Node.
    Server,
    /// Desktop apps: FreeSynergy binaries, Bots, Messenger adapters.
    Apps,
    /// Desktop customisation: Themes, Widgets, Cursors, Icons, Languages.
    Desktop,
}

impl StoreSection {
    pub fn id(&self) -> &'static str {
        match self { Self::Server => "Server", Self::Apps => "Apps", Self::Desktop => "Desktop" }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Server  => fs_i18n::t("store.section_tab.server").to_string(),
            Self::Apps    => fs_i18n::t("store.section_tab.apps").to_string(),
            Self::Desktop => fs_i18n::t("store.section_tab.desktop").to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self { Self::Server => ICON_SERVER, Self::Apps => ICON_APPS, Self::Desktop => ICON_DESKTOP }
    }

    pub fn from_id(id: &str) -> Self {
        match id { "Apps" => Self::Apps, "Desktop" => Self::Desktop, _ => Self::Server }
    }

    /// Sidebar items for this section (excludes the pinned Settings entry).
    pub fn sidebar_items(&self) -> Vec<SidebarItem> {
        match self {
            Self::Server => vec![
                SidebarItem::new("server",  ICON_SERVER,  fs_i18n::t("store.sidebar.server")),
                SidebarItem::new("bridges", ICON_BRIDGE,  fs_i18n::t("store.sidebar.bridges")),
            ],
            Self::Apps => vec![
                SidebarItem::new("bundles",  ICON_BUNDLE, fs_i18n::t("store.sidebar.bundles")),
                SidebarItem::new("apps",     ICON_APPS,   fs_i18n::t("store.sidebar.apps")),
                SidebarItem::new("bots",     ICON_BOT,    fs_i18n::t("store.sidebar.bots")),
                SidebarItem::new("installed",ICON_INSTALLED, fs_i18n::t("store.sidebar.installed")),
                SidebarItem::new("updates",  ICON_UPDATES, fs_i18n::t("store.sidebar.updates")),
            ],
            Self::Desktop => vec![
                SidebarItem::new("themes",   ICON_THEME,   fs_i18n::t("store.sidebar.themes")),
                SidebarItem::new("widgets",  ICON_WIDGET,  fs_i18n::t("store.sidebar.widgets")),
                SidebarItem::new("cursors",  ICON_CURSOR,  fs_i18n::t("store.sidebar.cursors")),
                SidebarItem::new("icons",    ICON_ICONS,   fs_i18n::t("store.sidebar.icons")),
                SidebarItem::new("languages",ICON_LANG,    fs_i18n::t("store.sidebar.languages")),
                SidebarItem::new("installed",ICON_INSTALLED, fs_i18n::t("store.sidebar.installed")),
                SidebarItem::new("updates",  ICON_UPDATES, fs_i18n::t("store.sidebar.updates")),
            ],
        }
    }

    /// Default sidebar item ID for this section.
    pub fn default_item(&self) -> &'static str {
        match self { Self::Server => "server", Self::Apps => "apps", Self::Desktop => "themes" }
    }

    /// Map a sidebar item ID to the PackageKind filter for this section.
    /// Returns `None` (show all) for items that cover all kinds.
    pub fn kinds_for(&self, item_id: &str) -> Vec<PackageKind> {
        match (self, item_id) {
            (_, "bundles")    => vec![PackageKind::Bundle],
            (Self::Server, "server")   => vec![PackageKind::Container],
            (Self::Server, "bridges")  => vec![PackageKind::Bridge],
            (Self::Apps, "apps")       => vec![PackageKind::App, PackageKind::Manager],
            (Self::Apps, "bots")       => vec![PackageKind::BotCommand, PackageKind::MessengerAdapter],
            (Self::Desktop, "themes")  => vec![PackageKind::Theme],
            (Self::Desktop, "widgets") => vec![PackageKind::Widget],
            // Cursors and Icons are Theme sub-types until dedicated PackageKinds are added.
            (Self::Desktop, "cursors") => vec![PackageKind::Theme],
            (Self::Desktop, "icons")   => vec![PackageKind::Theme],
            (Self::Desktop, "languages") => vec![PackageKind::Language],
            _ => vec![],
        }
    }
}

// ── StoreApp ─────────────────────────────────────────────────────────────────

/// Root Store component.
///
/// Layout:
/// ```
/// ┌─────────────────────────────────────────────────────┐
/// │  [Server]  [Apps]  [Desktop]     ← FsTabView bar   │
/// ├──────────────────────────────────────────────────────┤
/// │  ┌───────────┐  ┌────────────────────────────────┐  │
/// │  │ Bundles   │  │  search …                      │  │
/// │  │ Server    │  │                                │  │
/// │  │ Bridges   │  │  package grid                  │  │
/// │  ├───────────┤  │                                │  │
/// │  │ ⚙ Settings│  │                                │  │
/// │  └───────────┘  └────────────────────────────────┘  │
/// └─────────────────────────────────────────────────────┘
/// ```
#[component]
pub fn StoreApp() -> Element {
    let mut section      = use_signal(|| StoreSection::Server);
    let mut active_item  = use_signal(|| StoreSection::Server.default_item().to_string());
    let mut search       = use_signal(String::new);
    let mut detail: Signal<Option<PackageEntry>> = use_signal(|| None);

    let catalog_versions: Signal<Vec<(String, String)>> = use_signal(Vec::new);
    {
        let catalog_versions = catalog_versions.clone();
        use_future(move || {
            let mut catalog_versions = catalog_versions.clone();
            async move {
                if let Ok(catalog) = StoreClient::node_store()
                    .fetch_catalog::<NodePackage>("Node", false)
                    .await
                {
                    catalog_versions.set(
                        catalog.packages.into_iter()
                            .map(|p| (p.id, p.version))
                            .collect(),
                    );
                }
            }
        });
    }

    // Show detail view when a package is selected.
    if let Some(pkg) = detail.read().clone() {
        return rsx! {
            PackageDetail {
                package: pkg,
                on_back: move |_| detail.set(None),
            }
        };
    }

    let cur_section = section.read().clone();
    let cur_item    = active_item.read().clone();

    // When the section changes, reset the active item to the section default.
    let section_tabs: Vec<FsTabDef> = vec![
        FsTabDef::new(StoreSection::Server.id(),  StoreSection::Server.label())
            .with_icon(ICON_SERVER.to_string()),
        FsTabDef::new(StoreSection::Apps.id(),    StoreSection::Apps.label())
            .with_icon(ICON_APPS.to_string()),
        FsTabDef::new(StoreSection::Desktop.id(), StoreSection::Desktop.label())
            .with_icon(ICON_DESKTOP.to_string()),
    ];

    let sidebar_items = cur_section.sidebar_items();
    let pinned = vec![
        SidebarItem::new("settings", ICON_SETTINGS, fs_i18n::t("store.sidebar.settings")),
    ];

    let kinds = cur_section.kinds_for(&cur_item);

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        style { "{FS_TAB_VIEW_CSS}" }
        div {
            class: "fs-store",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fs-color-bg-base);",

            // Section tab bar + animated content
            FsTabView {
                tabs:      section_tabs,
                active_id: cur_section.id().to_string(),
                on_change: move |id: String| {
                    let sec = StoreSection::from_id(&id);
                    let def = sec.default_item().to_string();
                    section.set(sec);
                    active_item.set(def);
                    search.set(String::new());
                },
                // ── Section body (sidebar + content) ──────────────────────────
                div {
                    style: "display: flex; flex: 1; overflow: hidden; height: 100%;",

                    // Left sidebar (section-specific, Settings pinned)
                    Sidebar {
                        items:        sidebar_items,
                        pinned_items: pinned,
                        active_id:    cur_item.clone(),
                        on_select:    move |id: String| active_item.set(id),
                    }

                    // Right content
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                        // Search bar (not shown for settings, installed, updates)
                        if !matches!(cur_item.as_str(), "settings" | "installed" | "updates") {
                            div {
                                style: "padding: 12px 16px; background: var(--fs-color-bg-surface); \
                                        border-bottom: 1px solid var(--fs-color-border-default); flex-shrink: 0;",
                                input {
                                    r#type:      "search",
                                    placeholder: fs_i18n::t("store.search_placeholder").to_string(),
                                    style: "width: 100%; padding: 7px 12px; \
                                            border: 1px solid var(--fs-color-border-default); \
                                            border-radius: var(--fs-radius-md); font-size: 14px; \
                                            background: var(--fs-bg-input); color: var(--fs-text-primary);",
                                    oninput: move |e| *search.write() = e.value(),
                                    value: search.read().clone(),
                                }
                            }
                        }

                        // Content area
                        div {
                            style: "flex: 1; overflow: auto; padding: 16px;",
                            match cur_item.as_str() {
                                "installed" => rsx! {
                                    InstalledList {
                                        catalog_versions: catalog_versions.read().clone(),
                                    }
                                },
                                "updates" => rsx! {
                                    UpdatesList {
                                        catalog_versions: catalog_versions.read().clone(),
                                    }
                                },
                                "settings" => rsx! { StoreSettings {} },
                                _ => rsx! {
                                    PackageBrowser {
                                        search: search.read().clone(),
                                        kinds,
                                        on_select: move |pkg| detail.set(Some(pkg)),
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UpdatesList(catalog_versions: Vec<(String, String)>) -> Element {
    rsx! {
        div {
            style: "text-align: center; color: var(--fs-text-muted); padding: 48px;",
            p { style: "font-size: 24px; margin-bottom: 12px;", "↑" }
            p { style: "margin-bottom: 8px;", {fs_i18n::t("store.updates.no_metadata")} }
            p { style: "font-size: 13px;",
                "Run "
                code { style: "background: var(--fs-bg-elevated); padding: 2px 6px; border-radius: 4px;",
                    "fsn deploy"
                }
                " "
                {fs_i18n::t("store.updates.run_deploy")}
            }
            if !catalog_versions.is_empty() {
                p { style: "margin-top: 16px; font-size: 13px;",
                    {fs_i18n::t_with("store.updates.catalog_count",
                        &[("n", catalog_versions.len().to_string().as_str())])}
                }
            }
        }
    }
}
