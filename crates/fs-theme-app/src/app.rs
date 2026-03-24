/// Theme Manager — main app component.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, FS_SIDEBAR_CSS};
use fs_i18n;

use crate::chrome_view::ChromeView;
use crate::colors_view::ColorsView;
use crate::cursor_view::CursorView;
use crate::themes_view::ThemesView;

/// Active section in the Theme Manager.
#[derive(Clone, PartialEq, Debug)]
pub enum ThemeSection {
    Themes,
    Colors,
    Cursor,
    Chrome,
}

/// Display properties for a `ThemeSection` variant — single source of truth.
struct SectionMeta {
    id:       &'static str,
    label:    &'static str,
    icon:     &'static str,
}

const ALL_SECTIONS: &[ThemeSection] = &[
    ThemeSection::Themes,
    ThemeSection::Colors,
    ThemeSection::Cursor,
    ThemeSection::Chrome,
];

impl ThemeSection {
    /// Single match block — all display properties in one place.
    fn meta(&self) -> SectionMeta {
        match self {
            Self::Themes => SectionMeta { id: "themes", label: "theme.section.themes", icon: "🎨" },
            Self::Colors => SectionMeta { id: "colors", label: "theme.section.colors", icon: "🖌" },
            Self::Cursor => SectionMeta { id: "cursor", label: "theme.section.cursor", icon: "🖱" },
            Self::Chrome => SectionMeta { id: "chrome", label: "theme.section.chrome", icon: "🪟" },
        }
    }

    pub fn id(&self)    -> &str    { self.meta().id }
    pub fn icon(&self)  -> &str    { self.meta().icon }
    pub fn label(&self) -> String  { fs_i18n::t(self.meta().label).to_string() }

    /// No match needed — delegates to `id()` via ALL_SECTIONS.
    pub fn from_id(id: &str) -> Option<Self> {
        ALL_SECTIONS.iter().find(|s| s.id() == id).cloned()
    }
}

/// Root component of the Theme Manager.
#[component]
pub fn ThemeManagerApp() -> Element {
    let mut active = use_signal(|| ThemeSection::Themes);

    let sidebar_items: Vec<SidebarItem> = ALL_SECTIONS.iter()
        .map(|s| SidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        div {
            class: "fs-theme-manager",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fs-border); \
                        flex-shrink: 0; background: var(--fs-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fs-text-primary);",
                    {fs_i18n::t("theme.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                Sidebar {
                    items:     sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = ThemeSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                div {
                    style: "flex: 1; overflow: auto; padding: 20px;",
                    match *active.read() {
                        ThemeSection::Themes => rsx! { ThemesView {} },
                        ThemeSection::Colors => rsx! { ColorsView {} },
                        ThemeSection::Cursor => rsx! { CursorView {} },
                        ThemeSection::Chrome => rsx! { ChromeView {} },
                    }
                }
            }
        }
    }
}
