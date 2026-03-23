/// Builder — root component: Container builder, Bridge builder, i18n editor, resource browser.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, FS_SIDEBAR_CSS};

use crate::bridge_builder::BridgeBuilder;
use crate::container_builder::ContainerBuilder;
use crate::i18n_editor::I18nEditor;
use crate::resource_browser::ResourceBrowser;

#[derive(Clone, PartialEq, Debug)]
pub enum BuilderTab {
    Container,
    Bridge,
    I18n,
    Resources,
}

impl BuilderTab {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Container => "container-app",
            Self::Bridge       => "bridge",
            Self::I18n         => "i18n",
            Self::Resources    => "resources",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Container => "Container",
            Self::Bridge       => "Bridge",
            Self::I18n         => "i18n Editor",
            Self::Resources    => "Resources",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Container => "📦",
            Self::Bridge       => "🔗",
            Self::I18n         => "🌐",
            Self::Resources    => "📁",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "container-app" => Some(Self::Container),
            "bridge"        => Some(Self::Bridge),
            "i18n"          => Some(Self::I18n),
            "resources"     => Some(Self::Resources),
            _               => None,
        }
    }
}

const ALL_TABS: &[BuilderTab] = &[
    BuilderTab::Container,
    BuilderTab::Bridge,
    BuilderTab::I18n,
    BuilderTab::Resources,
];

/// Root Builder component.
#[component]
pub fn BuilderApp() -> Element {
    let mut active_tab = use_signal(|| BuilderTab::Container);

    let sidebar_items: Vec<SidebarItem> = ALL_TABS.iter()
        .map(|t| SidebarItem::new(t.id(), t.icon(), t.label()))
        .collect();

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        div {
            class: "fs-builder",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fs-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fs-border); \
                        flex-shrink: 0; background: var(--fs-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fs-text-primary);",
                    "Builder"
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                Sidebar {
                    items: sidebar_items,
                    active_id: active_tab.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(tab) = BuilderTab::from_id(&id) {
                            active_tab.set(tab);
                        }
                    },
                }

                // Content
                div {
                    style: "flex: 1; overflow: auto;",
                    match *active_tab.read() {
                        BuilderTab::Container => rsx! { ContainerBuilder {} },
                        BuilderTab::Bridge       => rsx! { BridgeBuilder {} },
                        BuilderTab::I18n         => rsx! { I18nEditor {} },
                        BuilderTab::Resources    => rsx! { ResourceBrowser {} },
                    }
                }
            }
        }
    }
}
