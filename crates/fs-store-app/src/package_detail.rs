/// Package detail panel — shown when the user clicks a package card.
///
/// Displays icon, name, description, tags, capability badges, metadata, and an
/// Install/Remove button. Uses InstallPopup for install result feedback.
use dioxus::prelude::*;
use fs_db_desktop::package_registry::PackageRegistry;

use crate::install_wizard::{do_install, InstallPopup, InstallResult};
use crate::missing_icon::MissingIcon;
use crate::node_package::PackageKind;
use crate::package_card::PackageEntry;
use crate::state::notify_install_changed;

// ── PackageDetail ─────────────────────────────────────────────────────────────

/// Full-page or side-panel detail view for a single package.
#[component]
pub fn PackageDetail(
    package: PackageEntry,
    on_back: EventHandler<()>,
    #[props(default)]
    on_select_package: Option<EventHandler<String>>,
) -> Element {
    let mut installing:     Signal<bool>                  = use_signal(|| false);
    let mut install_result: Signal<Option<InstallResult>> = use_signal(|| None);
    let mut installed       = use_signal(|| PackageRegistry::is_installed(&package.id));
    let mut remove_confirm  = use_signal(|| false);

    rsx! {
        div {
            class: "fs-package-detail fs-page-fade",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fs-color-bg-base);",

            // Install result popup
            if let Some(result) = install_result.read().clone() {
                {
                    let pkg_id = package.id.clone();
                    rsx! {
                        InstallPopup {
                            result,
                            on_close: move |_| {
                                installed.set(PackageRegistry::is_installed(&pkg_id));
                                install_result.set(None);
                            },
                        }
                    }
                }
            }

            // Remove confirm dialog
            if *remove_confirm.read() {
                div {
                    style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); \
                            display: flex; align-items: center; justify-content: center; z-index: 1000;",
                    div {
                        style: "background: var(--fs-color-bg-surface); \
                                border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-lg); padding: 24px; \
                                max-width: 400px; width: 100%;",
                        h3 { style: "margin: 0 0 12px 0;",
                            {fs_i18n::t_with("store.dialog.remove_title", &[("name", package.name.as_str())])}
                        }
                        p {
                            style: "color: var(--fs-color-text-muted); font-size: 14px; margin-bottom: 20px;",
                            {fs_i18n::t("store.dialog.remove_body")}
                        }
                        div {
                            style: "display: flex; gap: 8px; justify-content: flex-end;",
                            button {
                                style: "padding: 8px 16px; background: var(--fs-color-bg-surface); \
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); cursor: pointer;",
                                onclick: move |_| remove_confirm.set(false),
                                {fs_i18n::t("actions.cancel")}
                            }
                            button {
                                style: "padding: 8px 16px; background: var(--fs-color-error, #ef4444); \
                                        color: white; border: none; \
                                        border-radius: var(--fs-radius-md); cursor: pointer;",
                                onclick: {
                                    let pkg_id   = package.id.clone();
                                    let is_bundle = package.kind == PackageKind::Bundle;
                                    move |_| {
                                        if is_bundle {
                                            let _ = PackageRegistry::remove_bundle(&pkg_id);
                                        } else {
                                            let _ = PackageRegistry::remove(&pkg_id);
                                        }
                                        notify_install_changed();
                                        installed.set(false);
                                        remove_confirm.set(false);
                                    }
                                },
                                {fs_i18n::t("actions.remove")}
                            }
                        }
                    }
                }
            }

            // ── Top bar ──────────────────────────────────────────────────────
            div {
                style: "display: flex; align-items: center; gap: 12px; padding: 14px 20px; \
                        border-bottom: 1px solid var(--fs-color-border-default); \
                        background: var(--fs-color-bg-surface);",
                button {
                    style: "background: none; border: none; cursor: pointer; \
                            color: var(--fs-color-text-muted); font-size: 20px; padding: 0 4px;",
                    title: fs_i18n::t("store.back_to_browser").to_string(),
                    onclick: move |_| on_back.call(()),
                    "‹"
                }
                span {
                    style: "font-size: 14px; color: var(--fs-color-text-muted);",
                    {fs_i18n::t("store.title")}
                }
                span { style: "color: var(--fs-color-text-muted);", "/" }
                span {
                    style: "font-size: 14px; color: var(--fs-color-text-primary);",
                    "{package.name}"
                }
            }

            // ── Scrollable body ──────────────────────────────────────────────
            div {
                style: "flex: 1; overflow-y: auto; padding: 32px;",

                // Header row: icon + title + meta
                div {
                    style: "display: flex; align-items: flex-start; gap: 24px; margin-bottom: 32px;",

                    // Icon
                    div {
                        style: "width: 72px; height: 72px; flex-shrink: 0; \
                                border-radius: var(--fs-radius-lg); \
                                background: var(--fs-color-bg-surface); \
                                border: 1px solid var(--fs-color-border-default); \
                                display: flex; align-items: center; justify-content: center; \
                                font-size: 36px;",
                        if let Some(icon_url) = &package.icon {
                            img {
                                src: "{icon_url}",
                                width: "48",
                                height: "48",
                                style: "object-fit: contain;",
                            }
                        } else {
                            MissingIcon { size: 48 }
                        }
                    }

                    // Title + meta
                    div { style: "flex: 1;",
                        h2 { style: "margin: 0 0 4px 0; font-size: 24px;", "{package.name}" }
                        p {
                            style: "margin: 0 0 12px 0; font-size: 14px; \
                                    color: var(--fs-color-text-muted);",
                            "v{package.version} · {package.category}"
                        }
                        // Kind badge + tags
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 6px;",
                            span {
                                style: "padding: 2px 10px; border-radius: 999px; font-size: 12px; \
                                        background: var(--fs-color-primary); color: white;",
                                "{package.kind.label()}"
                            }
                            for tag in &package.tags {
                                span {
                                    key: "{tag}",
                                    style: "padding: 2px 10px; border-radius: 999px; font-size: 12px; \
                                            background: var(--fs-color-bg-surface); \
                                            border: 1px solid var(--fs-color-border-default); \
                                            color: var(--fs-color-text-muted);",
                                    "{tag}"
                                }
                            }
                        }
                    }

                    // Install / Remove button (top-right)
                    div { style: "display: flex; flex-direction: column; gap: 8px; align-items: flex-end;",
                        if *installed.read() {
                            button {
                                style: "padding: 10px 24px; background: var(--fs-color-bg-overlay); \
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); cursor: default; \
                                        font-size: 14px; color: var(--fs-color-text-muted);",
                                disabled: true,
                                {fs_i18n::t("store.status.installed")}
                            }
                            // Only show Remove if NOT installed via a bundle
                            if package.installed_by.is_none() {
                                button {
                                    style: "padding: 6px 16px; background: transparent; \
                                            border: 1px solid var(--fs-color-error, #ef4444); \
                                            color: var(--fs-color-error, #ef4444); \
                                            border-radius: var(--fs-radius-md); cursor: pointer; \
                                            font-size: 12px;",
                                    onclick: move |_| remove_confirm.set(true),
                                    {fs_i18n::t("actions.remove")}
                                }
                            } else {
                                span {
                                    style: "font-size: 11px; color: var(--fs-color-text-muted);",
                                    {fs_i18n::t_with("store.status.installed_via_bundle",
                                        &[("bundle", package.installed_by.as_deref().unwrap_or(""))])}
                                }
                            }
                        } else if *installing.read() {
                            button {
                                style: "padding: 10px 24px; background: var(--fs-color-bg-overlay); \
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); cursor: default; \
                                        font-size: 14px; color: var(--fs-color-text-muted);",
                                disabled: true,
                                "⏳ Installing…"
                            }
                        } else {
                            button {
                                style: "padding: 10px 24px; background: var(--fs-color-primary); \
                                        color: white; border: none; \
                                        border-radius: var(--fs-radius-md); cursor: pointer; \
                                        font-size: 14px; font-weight: 600;",
                                onclick: {
                                    let pkg = package.clone();
                                    move |_| {
                                        let pkg2 = pkg.clone();
                                        installing.set(true);
                                        spawn(async move {
                                            let result = match do_install(pkg2, String::new()).await {
                                                Ok(()) => {
                                                    notify_install_changed();
                                                    InstallResult::Success
                                                }
                                                Err(e) => InstallResult::Failed(e),
                                            };
                                            installing.set(false);
                                            install_result.set(Some(result));
                                        });
                                    }
                                },
                                {fs_i18n::t("actions.install")}
                            }
                        }
                    }
                }

                // Description
                div { style: "margin-bottom: 32px;",
                    h3 {
                        style: "font-size: 14px; font-weight: 600; \
                                text-transform: uppercase; letter-spacing: 0.06em; \
                                color: var(--fs-color-text-muted); margin: 0 0 12px 0;",
                        {fs_i18n::t("labels.description")}
                    }
                    p {
                        style: "font-size: 14px; line-height: 1.7; \
                                color: var(--fs-color-text-secondary);",
                        "{package.description}"
                    }
                }

                // Bundle: Included Packages section
                if package.kind == PackageKind::Bundle && !package.capabilities.is_empty() {
                    div { style: "margin-bottom: 32px;",
                        h3 {
                            style: "font-size: 14px; font-weight: 600; \
                                    text-transform: uppercase; letter-spacing: 0.06em; \
                                    color: var(--fs-color-text-muted); margin: 0 0 12px 0;",
                            "Included Packages"
                        }
                        div { style: "display: flex; flex-direction: column; gap: 6px;",
                            for cap in &package.capabilities {
                                {
                                    let cap_id      = cap.clone();
                                    let has_handler = on_select_package.is_some();
                                    let is_member_installed = PackageRegistry::is_installed(&cap_id);
                                    rsx! {
                                        div {
                                            key: "{cap_id}",
                                            style: if has_handler {
                                                "display: flex; align-items: center; gap: 8px; \
                                                 padding: 8px 14px; \
                                                 background: var(--fs-color-bg-surface); \
                                                 border: 1px solid var(--fs-color-border-default); \
                                                 border-radius: var(--fs-radius-md); \
                                                 font-size: 13px; cursor: pointer;"
                                            } else {
                                                "display: flex; align-items: center; gap: 8px; \
                                                 padding: 8px 14px; \
                                                 background: var(--fs-color-bg-surface); \
                                                 border: 1px solid var(--fs-color-border-default); \
                                                 border-radius: var(--fs-radius-md); font-size: 13px;"
                                            },
                                            onclick: move |_| {
                                                if let Some(ref handler) = on_select_package {
                                                    handler.call(cap_id.clone());
                                                }
                                            },
                                            span {
                                                style: "color: var(--fs-color-text-muted);",
                                                if is_member_installed { "✅" } else { "○" }
                                            }
                                            span { style: "flex: 1;", "{cap}" }
                                            if is_member_installed {
                                                span {
                                                    style: "font-size: 11px; color: var(--fs-color-success, #22c55e); \
                                                            background: rgba(34,197,94,0.12); \
                                                            padding: 1px 6px; border-radius: 999px;",
                                                    {fs_i18n::t("store.status.installed")}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Capabilities
                if package.kind != PackageKind::Bundle && !package.capabilities.is_empty() {
                    div { style: "margin-bottom: 32px;",
                        h3 {
                            style: "font-size: 14px; font-weight: 600; \
                                    text-transform: uppercase; letter-spacing: 0.06em; \
                                    color: var(--fs-color-text-muted); margin: 0 0 12px 0;",
                            {fs_i18n::t("store.section.capabilities")}
                        }
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 8px;",
                            for cap in &package.capabilities {
                                CapabilityBadge { label: cap.clone() }
                            }
                        }
                    }
                }

                // Metadata table
                div {
                    h3 {
                        style: "font-size: 14px; font-weight: 600; \
                                text-transform: uppercase; letter-spacing: 0.06em; \
                                color: var(--fs-color-text-muted); margin: 0 0 12px 0;",
                        {fs_i18n::t("store.section.package_info")}
                    }
                    div {
                        style: "background: var(--fs-color-bg-surface); \
                                border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); overflow: hidden;",
                        MetaRow { label: fs_i18n::t("labels.id"),      value: package.id.clone() }
                        MetaRow { label: fs_i18n::t("labels.version"), value: package.version.clone() }
                        MetaRow { label: "Type".to_string(),            value: package.kind.label().to_string() }
                        MetaRow { label: "Category".to_string(),        value: package.category.clone() }
                        if !package.author.is_empty() {
                            MetaRow { label: "Author".to_string(), value: package.author.clone() }
                        }
                        if !package.license.is_empty() {
                            MetaRow { label: "License".to_string(), value: package.license.clone() }
                        }
                        if let Some(ref path) = package.store_path {
                            MetaRow { label: "Path".to_string(), value: path.clone() }
                        }
                    }
                }
            }
        }
    }
}

// ── CapabilityBadge ───────────────────────────────────────────────────────────

#[component]
fn CapabilityBadge(label: String) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 6px; \
                    padding: 6px 14px; \
                    background: var(--fs-color-bg-surface); \
                    border: 1px solid var(--fs-color-border-default); \
                    border-radius: var(--fs-radius-md); font-size: 13px;",
            span { "✦" }
            span { "{label}" }
        }
    }
}

// ── MetaRow ───────────────────────────────────────────────────────────────────

#[component]
fn MetaRow(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; padding: 10px 16px; \
                    border-bottom: 1px solid var(--fs-color-border-default); \
                    font-size: 13px;",
            span {
                style: "min-width: 100px; color: var(--fs-color-text-muted);",
                "{label}"
            }
            span {
                style: "color: var(--fs-color-text-primary);",
                "{value}"
            }
        }
    }
}
