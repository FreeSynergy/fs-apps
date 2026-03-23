/// Container App Manager — manage running/stopped containers, browse store, build packages.
use dioxus::prelude::*;
use fs_components::{AppContext, Sidebar, SidebarItem, FS_SIDEBAR_CSS};
use fs_i18n;

use crate::build_view::BuildView;
use crate::log_viewer::LogViewer;
use crate::service_detail::ServiceDetail;
use crate::service_list::ServiceList;

/// Display properties for a `ContainerSection` variant — single source of truth.
struct SectionMeta {
    id:       &'static str,
    icon:     &'static str,
    i18n_key: &'static str,
}

/// Active section in the Container App Manager.
#[derive(Clone, PartialEq, Debug)]
pub enum ContainerSection {
    Installed,
    InstallNew,
    Build,
    Logs,
}

const ALL_SECTIONS: &[ContainerSection] = &[
    ContainerSection::Installed,
    ContainerSection::InstallNew,
    ContainerSection::Build,
    ContainerSection::Logs,
];

impl ContainerSection {
    /// Single match block — all display properties in one place.
    fn meta(&self) -> SectionMeta {
        match self {
            Self::Installed  => SectionMeta { id: "installed",   icon: "📦", i18n_key: "container.section.installed"   },
            Self::InstallNew => SectionMeta { id: "install_new", icon: "🛍", i18n_key: "container.section.install_new" },
            Self::Build      => SectionMeta { id: "build",       icon: "🔧", i18n_key: "container.section.build"       },
            Self::Logs       => SectionMeta { id: "logs",        icon: "📋", i18n_key: "container.section.logs"        },
        }
    }

    pub fn id(&self)    -> &str   { self.meta().id }
    pub fn icon(&self)  -> &str   { self.meta().icon }
    pub fn label(&self) -> String { fs_i18n::t(self.meta().i18n_key).to_string() }

    /// No match needed — delegates to `id()` via ALL_SECTIONS.
    pub fn from_id(id: &str) -> Option<Self> {
        ALL_SECTIONS.iter().find(|s| s.id() == id).cloned()
    }
}

/// Root component of the Container App Manager.
#[component]
pub fn Container() -> Element {
    let mut active = use_signal(|| ContainerSection::Installed);
    let mut selected_service: Signal<Option<String>> = use_signal(|| None);

    let sidebar_items: Vec<SidebarItem> = ALL_SECTIONS.iter()
        .map(|s| SidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        div {
            class: "fs-container-app",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fs-border); \
                        flex-shrink: 0; background: var(--fs-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fs-text-primary);",
                    {fs_i18n::t("container.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                // ── Left nav sidebar ───────────────────────────────────────────────
                Sidebar {
                    items:     sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = ContainerSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                // ── Detail area ───────────────────────────────────────────────────
                div {
                    class: "fs-container-app__detail fs-page-enter",
                    style: "flex: 1; display: flex; overflow: hidden;",

                    match *active.read() {
                        // Installed: split-view — list on left, detail on right
                        ContainerSection::Installed => rsx! {
                            div {
                                style: {
                                    if selected_service.read().is_some() {
                                        "flex: 0 0 340px; overflow: auto; padding: 16px; \
                                         border-right: 1px solid var(--fs-border);"
                                    } else {
                                        "flex: 1; overflow: auto; padding: 16px;"
                                    }
                                },
                                ServiceList { selected: selected_service }
                            }
                            if let Some(name) = selected_service.read().clone() {
                                div { style: "flex: 1; overflow: hidden;",
                                    ServiceDetail {
                                        service_name: name,
                                        on_close: move |_| *selected_service.write() = None,
                                    }
                                }
                            }
                        },
                        ContainerSection::InstallNew => rsx! {
                            div {
                                style: "flex: 1; overflow: auto; padding: 40px; \
                                        display: flex; align-items: flex-start; justify-content: center;",
                                div {
                                    style: "max-width: 600px; width: 100%; text-align: center; \
                                            color: var(--fs-color-text-muted);",
                                    p { style: "font-size: 32px; margin-bottom: 16px;", "🛍" }
                                    h3 { style: "margin: 0 0 8px; color: var(--fs-color-text-primary);",
                                        "Browse Store"
                                    }
                                    p { style: "font-size: 13px;",
                                        "Discover and install new container apps from the FreeSynergy Store."
                                    }
                                    button {
                                        style: "margin-top: 16px; background: var(--fs-color-primary, #06b6d4); \
                                                color: #fff; border: none; border-radius: var(--fs-radius-md); \
                                                padding: 10px 24px; font-size: 14px; font-weight: 600; \
                                                cursor: pointer;",
                                        onclick: move |_| {
                                            if let Some(mut ctx) = try_use_context::<AppContext>() {
                                                ctx.app_open_req.set(Some("store".to_string()));
                                            }
                                        },
                                        "Open Store"
                                    }
                                }
                            }
                        },
                        ContainerSection::Build => rsx! {
                            div { style: "flex: 1; overflow: auto; padding: 16px;",
                                BuildView {}
                            }
                        },
                        ContainerSection::Logs => rsx! {
                            div { style: "flex: 1; overflow: auto; padding: 16px;",
                                LogViewer {
                                    service: selected_service.read().clone().unwrap_or_default()
                                }
                            }
                        },
                    }
                }
            } // end sidebar + content row
        }
    }
}
