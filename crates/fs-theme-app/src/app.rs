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

impl ThemeSection {
    pub fn id(&self) -> &str {
        match self {
            Self::Themes => "themes",
            Self::Colors => "colors",
            Self::Cursor => "cursor",
            Self::Chrome => "chrome",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Themes => fs_i18n::t("theme.section.themes").to_string(),
            Self::Colors => fs_i18n::t("theme.section.colors").to_string(),
            Self::Cursor => fs_i18n::t("theme.section.cursor").to_string(),
            Self::Chrome => fs_i18n::t("theme.section.chrome").to_string(),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Themes => "🎨",
            Self::Colors => "🖌",
            Self::Cursor => "🖱",
            Self::Chrome => "🪟",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "themes" => Some(Self::Themes),
            "colors" => Some(Self::Colors),
            "cursor" => Some(Self::Cursor),
            "chrome" => Some(Self::Chrome),
            _        => None,
        }
    }
}

const ALL_SECTIONS: &[ThemeSection] = &[
    ThemeSection::Themes,
    ThemeSection::Colors,
    ThemeSection::Cursor,
    ThemeSection::Chrome,
];

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
