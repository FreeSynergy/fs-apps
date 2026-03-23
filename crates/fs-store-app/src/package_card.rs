/// Package card — displays a single package in the store browser.
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::node_package::PackageKind;

/// A package entry from the FSN store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub kind: PackageKind,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub icon: Option<String>,
    /// Store-relative path to the package directory (used for file downloads).
    pub store_path: Option<String>,
    pub installed: bool,
    pub update_available: bool,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub author: String,
    /// Set if this package was installed as a member of a bundle (contains the bundle ID).
    #[serde(default)]
    pub installed_by: Option<String>,
}

use crate::missing_icon::MissingIcon;

/// Package card component.
#[component]
pub fn PackageCard(
    package:    PackageEntry,
    on_details: EventHandler<MouseEvent>,
    #[props(default)]
    on_install: Option<EventHandler<()>>,
    #[props(default)]
    on_remove:  Option<EventHandler<()>>,
) -> Element {
    let mut show_dropdown  = use_signal(|| false);
    let mut confirm_remove = use_signal(|| false);

    rsx! {
        div {
            class: "fs-package-card",
            style: "background: var(--fs-color-bg-surface); border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-lg); padding: 16px; display: flex; flex-direction: column; \
                    gap: 8px; transition: border-color 0.15s; position: relative;",

            // Remove confirmation dialog (fixed so it's always visible)
            if *confirm_remove.read() {
                div {
                    style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); \
                            display: flex; align-items: center; justify-content: center; z-index: 9000;",
                    onclick: move |_| confirm_remove.set(false),
                    div {
                        style: "background: var(--fs-color-bg-surface); \
                                border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-lg); padding: 24px; \
                                max-width: 380px; width: 100%; display: flex; flex-direction: column; gap: 16px;",
                        onclick: move |evt: MouseEvent| evt.stop_propagation(),
                        p {
                            style: "font-size: 15px; font-weight: 600; color: var(--fs-color-text-primary); margin: 0;",
                            {fs_i18n::t_with("store.dialog.remove_title", &[("name", package.name.as_str())])}
                        }
                        p {
                            style: "font-size: 13px; color: var(--fs-color-text-muted); margin: 0;",
                            {fs_i18n::t("store.dialog.remove_body")}
                        }
                        div {
                            style: "display: flex; gap: 8px; justify-content: flex-end;",
                            button {
                                style: "padding: 7px 16px; border-radius: var(--fs-radius-md); \
                                        border: 1px solid var(--fs-color-border-default); \
                                        background: transparent; color: var(--fs-color-text-primary); \
                                        cursor: pointer; font-size: 13px;",
                                onclick: move |_| confirm_remove.set(false),
                                {fs_i18n::t("actions.cancel")}
                            }
                            button {
                                style: "padding: 7px 16px; border-radius: var(--fs-radius-md); \
                                        border: none; background: var(--fs-color-error, #ef4444); \
                                        color: white; cursor: pointer; font-size: 13px;",
                                onclick: {
                                    let on_remove = on_remove.clone();
                                    move |_| {
                                        confirm_remove.set(false);
                                        if let Some(ref h) = on_remove {
                                            h.call(());
                                        }
                                    }
                                },
                                {fs_i18n::t("actions.remove")}
                            }
                        }
                    }
                }
            }

            // Header
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                div {
                    style: "width: 40px; height: 40px; border-radius: var(--fs-radius-md); background: var(--fs-color-bg-overlay); display: flex; align-items: center; justify-content: center; font-size: 20px;",
                    if let Some(icon) = &package.icon {
                        if icon.trim_start().starts_with("<svg") {
                            span {
                                style: "width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;",
                                dangerous_inner_html: "{icon}",
                            }
                        } else {
                            img {
                                src: "{icon}",
                                width: "32",
                                height: "32",
                                style: "object-fit: contain;",
                            }
                        }
                    } else {
                        MissingIcon { size: 32 }
                    }
                }
                div {
                    strong { style: "display: block; font-size: 15px;", "{package.name}" }
                    span { style: "font-size: 12px; color: var(--fs-color-text-muted);", "v{package.version} · {package.category}" }
                }
                if let Some(ref bundle_id) = package.installed_by {
                    span {
                        title: "Managed by bundle: {bundle_id}",
                        style: "margin-left: auto; font-size: 11px; color: var(--fs-color-text-muted); \
                                background: var(--fs-color-bg-overlay); padding: 2px 8px; \
                                border-radius: 999px; border: 1px solid var(--fs-color-border-default);",
                        {fs_i18n::t("store.status.managed_by_bundle")}
                    }
                } else if package.installed {
                    span {
                        style: "margin-left: auto; font-size: 11px; color: var(--fs-color-success, #22c55e); \
                                background: rgba(34,197,94,0.12); padding: 2px 8px; border-radius: 999px;",
                        {fs_i18n::t("store.status.installed")}
                    }
                }
            }

            // Description
            p {
                style: "margin: 0; font-size: 13px; color: var(--fs-color-text-secondary); line-height: 1.4;",
                "{package.description}"
            }

            // Tags
            if !package.tags.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 4px;",
                    for tag in package.tags.iter().take(5) {
                        span {
                            key: "{tag}",
                            style: "font-size: 10px; padding: 2px 6px; border-radius: 999px; \
                                    background: var(--fs-color-bg-overlay); \
                                    color: var(--fs-color-text-muted); \
                                    border: 1px solid var(--fs-color-border-default);",
                            "{tag}"
                        }
                    }
                }
            }

            // Button row
            div {
                style: "margin-top: auto; display: flex; gap: 8px;",

                // Details button
                button {
                    style: "flex: 1; padding: 8px; background: var(--fs-color-bg-overlay); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px; \
                            color: var(--fs-color-text-primary);",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_details.call(evt);
                    },
                    {fs_i18n::t("actions.details")}
                }

                // Action area — bundle-managed packages cannot be installed/removed individually
                if package.installed_by.is_some() {
                    span {
                        style: "flex: 1; padding: 8px; font-size: 12px; text-align: center; \
                                color: var(--fs-color-text-muted); \
                                border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md);",
                        {fs_i18n::t("store.status.managed_by_bundle")}
                    }
                } else if package.installed {
                    if package.update_available {
                        // Update button
                        button {
                            style: "flex: 1; padding: 8px; background: var(--fs-color-warning, #f59e0b); \
                                    color: white; border: none; \
                                    border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                            onclick: move |evt: MouseEvent| {
                                evt.stop_propagation();
                                if let Some(ref h) = on_install { h.call(()); }
                            },
                            {fs_i18n::t("store.status.update_available")}
                        }
                    } else {
                        // Reinstall | ▾ split button
                        div {
                            style: "flex: 1; display: flex; position: relative;",

                            // Reinstall (main action)
                            button {
                                style: "flex: 1; padding: 8px 10px; background: var(--fs-color-primary); \
                                        color: white; border: none; \
                                        border-radius: var(--fs-radius-md) 0 0 var(--fs-radius-md); \
                                        cursor: pointer; font-size: 13px;",
                                onclick: move |evt: MouseEvent| {
                                    evt.stop_propagation();
                                    if let Some(ref h) = on_install { h.call(()); }
                                },
                                {fs_i18n::t("actions.reinstall")}
                            }

                            // Dropdown toggle
                            button {
                                style: "padding: 8px 10px; background: var(--fs-color-primary); \
                                        color: white; border: none; border-left: 1px solid rgba(255,255,255,0.2); \
                                        border-radius: 0 var(--fs-radius-md) var(--fs-radius-md) 0; \
                                        cursor: pointer; font-size: 11px;",
                                onclick: move |evt: MouseEvent| {
                                    evt.stop_propagation();
                                    let cur = *show_dropdown.read();
                                    show_dropdown.set(!cur);
                                },
                                "▾"
                            }

                            // Dropdown menu
                            if *show_dropdown.read() {
                                // Backdrop to close on outside click
                                div {
                                    style: "position: fixed; inset: 0; z-index: 800;",
                                    onclick: move |_| show_dropdown.set(false),
                                }
                                div {
                                    style: "position: absolute; top: calc(100% + 4px); right: 0; \
                                            background: var(--fs-color-bg-surface); \
                                            border: 1px solid var(--fs-color-border-default); \
                                            border-radius: var(--fs-radius-md); \
                                            box-shadow: 0 4px 16px rgba(0,0,0,0.3); \
                                            min-width: 140px; overflow: hidden; z-index: 801;",
                                    button {
                                        style: "width: 100%; padding: 9px 14px; \
                                                background: var(--fs-color-error, #ef4444); \
                                                border: none; text-align: left; cursor: pointer; font-size: 13px; \
                                                color: white;",
                                        onclick: move |evt: MouseEvent| {
                                            evt.stop_propagation();
                                            show_dropdown.set(false);
                                            confirm_remove.set(true);
                                        },
                                        {fs_i18n::t("actions.remove")}
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Install button
                    button {
                        style: "flex: 1; padding: 8px; background: var(--fs-color-primary); \
                                color: white; border: none; \
                                border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                        onclick: move |evt: MouseEvent| {
                            evt.stop_propagation();
                            if let Some(ref h) = on_install { h.call(()); }
                        },
                        {fs_i18n::t("actions.install")}
                    }
                }
            }
        }
    }
}
