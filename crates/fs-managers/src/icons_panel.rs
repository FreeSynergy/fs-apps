/// Icons Manager panel — sidebar (icon sets) + detail pane (icons grid + search).
use dioxus::prelude::*;
use fs_i18n;
use fs_manager_icons::IconManager;

#[component]
pub fn IconsManagerPanel() -> Element {
    let icons_root = std::path::PathBuf::from(
        std::env::var("FS_ICONS_ROOT")
            .unwrap_or_else(|_| "../FreeSynergy.Icons".into())
    );
    let mgr  = IconManager::new(icons_root, vec![]);
    let sets = mgr.sets();

    let first_id = sets.first().map(|s| s.id.clone()).unwrap_or_default();
    let mut selected_id = use_signal(|| first_id);
    let mut search      = use_signal(String::new);

    let current_set = {
        let id = selected_id.read().clone();
        sets.iter().find(|s| s.id == id).cloned()
    };

    let icons = {
        let id  = selected_id.read().clone();
        let q   = search.read().clone();
        if id.is_empty() {
            vec![]
        } else {
            mgr.list_set(&id).unwrap_or_default()
                .into_iter()
                .filter(|name| q.is_empty() || name.to_lowercase().contains(&q.to_lowercase()))
                .collect::<Vec<_>>()
        }
    };

    rsx! {
        div {
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            // ── Left: Icon Set List ──────────────────────────────────────────
            div {
                style: "width: 220px; flex-shrink: 0; overflow-y: auto; \
                        background: var(--fs-color-bg-surface, #0f172a); \
                        border-right: 1px solid var(--fs-color-border-default, #334155); \
                        padding: 8px 0;",

                div {
                    style: "padding: 10px 16px 6px; font-size: 11px; font-weight: 600; \
                            letter-spacing: 0.05em; text-transform: uppercase; \
                            color: var(--fs-color-text-muted);",
                    {fs_i18n::t("managers.icons.sets_title")}
                }

                if sets.is_empty() {
                    div {
                        style: "padding: 12px 16px; font-size: 13px; \
                                color: var(--fs-color-text-muted);",
                        {fs_i18n::t("managers.icons.no_sets")}
                    }
                } else {
                    for set in &sets {
                        {
                            let set_id    = set.id.clone();
                            let is_active = *selected_id.read() == set_id;
                            let bg = if is_active {
                                "background: var(--fs-sidebar-active-bg, rgba(6,182,212,0.15)); \
                                 color: var(--fs-color-primary, #06b6d4);"
                            } else {
                                "background: transparent; color: var(--fs-color-text-primary);"
                            };
                            rsx! {
                                div {
                                    key: "{set_id}",
                                    style: "display: flex; align-items: center; gap: 10px; \
                                            padding: 9px 16px; cursor: pointer; \
                                            transition: background 100ms; {bg}",
                                    onclick: move |_| {
                                        selected_id.set(set_id.clone());
                                        search.set(String::new());
                                    },
                                    span { style: "font-size: 16px; flex-shrink: 0;", "🖼" }
                                    div { style: "flex: 1; min-width: 0;",
                                        div {
                                            style: "font-size: 13px; font-weight: 500; \
                                                    white-space: nowrap; overflow: hidden; \
                                                    text-overflow: ellipsis;",
                                            "{set.name}"
                                        }
                                        div {
                                            style: "font-size: 11px; opacity: 0.55; margin-top: 1px;",
                                            "{set.icon_count} icons"
                                        }
                                    }
                                    if set.builtin {
                                        span {
                                            style: "padding: 1px 5px; font-size: 9px; \
                                                    background: var(--fs-color-primary, #06b6d4); \
                                                    color: #fff; border-radius: 999px; flex-shrink: 0;",
                                            "✦"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Right: Icons Grid ────────────────────────────────────────────
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                // Header + Search
                div {
                    style: "flex-shrink: 0; padding: 16px 20px 12px; \
                            border-bottom: 1px solid var(--fs-color-border-default, #334155);",

                    if let Some(ref set) = current_set {
                        div {
                            style: "display: flex; align-items: baseline; gap: 12px; margin-bottom: 10px;",
                            h3 {
                                style: "margin: 0; font-size: 15px; font-weight: 600; \
                                        color: var(--fs-color-text-primary);",
                                "{set.name}"
                            }
                            if !set.description.is_empty() {
                                span {
                                    style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                    "{set.description}"
                                }
                            }
                        }
                    }

                    input {
                        style: "width: 100%; box-sizing: border-box; \
                                padding: 7px 12px; font-size: 13px; \
                                background: var(--fs-color-bg-overlay, #1e293b); \
                                border: 1px solid var(--fs-color-border-default, #334155); \
                                border-radius: var(--fs-radius-md, 6px); \
                                color: var(--fs-color-text-primary); outline: none;",
                        placeholder: "Search icons…",
                        value: "{search}",
                        oninput: move |e| search.set(e.value()),
                    }
                }

                // Icons grid
                div {
                    style: "flex: 1; overflow-y: auto; padding: 16px 20px;",

                    if icons.is_empty() {
                        div {
                            style: "display: flex; align-items: center; justify-content: center; \
                                    height: 120px; font-size: 13px; \
                                    color: var(--fs-color-text-muted);",
                            if selected_id.read().is_empty() {
                                {fs_i18n::t("managers.icons.select_set")}
                            } else {
                                {fs_i18n::t("managers.icons.no_icons")}
                            }
                        }
                    } else {
                        div {
                            style: "display: grid; \
                                    grid-template-columns: repeat(auto-fill, minmax(88px, 1fr)); \
                                    gap: 8px;",
                            for name in &icons {
                                {
                                    let set_id = selected_id.read().clone();
                                    let icon_path = {
                                        let icons_root2 = std::path::PathBuf::from(
                                            std::env::var("FS_ICONS_ROOT")
                                                .unwrap_or_else(|_| "../FreeSynergy.Icons".into())
                                        );
                                        let p = icons_root2.join(&set_id).join(format!("{name}.svg"));
                                        format!("file://{}", p.display())
                                    };
                                    rsx! {
                                        div {
                                            key: "{name}",
                                            style: "display: flex; flex-direction: column; \
                                                    align-items: center; gap: 5px; \
                                                    padding: 10px 6px; \
                                                    border-radius: var(--fs-radius-md, 6px); \
                                                    background: var(--fs-color-bg-overlay, #1e293b); \
                                                    cursor: default;",
                                            img {
                                                src: "{icon_path}",
                                                style: "width: 36px; height: 36px; object-fit: contain;",
                                                alt: "{name}",
                                            }
                                            span {
                                                style: "font-size: 10px; color: var(--fs-color-text-muted); \
                                                        text-align: center; word-break: break-all; \
                                                        max-width: 76px; line-height: 1.3;",
                                                "{name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
