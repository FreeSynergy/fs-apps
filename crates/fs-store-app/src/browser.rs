/// Package browser — fetches the live catalog and renders a filtered package grid.
use std::collections::HashMap;

use dioxus::prelude::*;
use fs_components::{LoadingOverlay, SpinnerSize};
use fs_db_desktop::package_registry::PackageRegistry;
use fs_store::StoreReader;

use crate::install_wizard::do_install;
use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::{PackageCard, PackageEntry};
use crate::state::{notify_install_changed, INSTALL_COUNTER};

/// Language codes that are always considered installed without needing a Store install.
/// English is the only truly built-in language — it is the fallback for all i18n lookups
/// and requires no installation. All other languages must be installed from the Store.
#[allow(dead_code)]
const BUILTIN_LANG_CODES: &[&str] = &["en"];

/// Install-state filter for the package browser.
#[derive(Clone, PartialEq, Debug)]
pub enum InstallFilter {
    All,
    Installed,
    Available,
    /// Only packages where `update_available` is true.
    Updatable,
}

/// Resolve an icon field to a usable URL, or `None` if it cannot be used as an `<img>` src.
///
/// Returns `Some(url)` only for:
/// - Absolute HTTP(S) URLs
/// - Relative paths (converted to raw.githubusercontent.com URLs)
///
/// Returns `None` for emoji glyphs, single characters, or empty strings so the
/// caller can fall back to the `MissingIcon` placeholder instead of a broken `<img>`.
pub fn resolve_icon_path(icon: &str) -> Option<String> {
    if icon.is_empty() {
        return None;
    }
    // Single Unicode scalar = emoji or single glyph — not a URL
    if icon.chars().count() <= 1 {
        return None;
    }
    if icon.starts_with("http") {
        return Some(icon.to_string());
    }
    if icon.starts_with('<') {
        // Inline SVG markup — not a path, skip for now
        return None;
    }
    // Relative path → prepend GitHub raw base
    Some(format!(
        "https://raw.githubusercontent.com/FreeSynergy/fs-store/main/{icon}"
    ))
}

/// Package browser component.
///
/// `kinds` filters by package type — empty slice means show all.
/// Multiple kinds are OR-combined.
#[component]
pub fn PackageBrowser(
    search: String,
    #[props(default)] kinds: Vec<PackageKind>,
    on_select: EventHandler<PackageEntry>,
) -> Element {
    let packages: Signal<Vec<PackageEntry>> = use_signal(Vec::new);
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut install_filter = use_signal(|| InstallFilter::All);

    // Refresh installed flags whenever a package is installed or removed.
    {
        let mut packages = packages;
        use_effect(move || {
            let _ = INSTALL_COUNTER.read(); // subscribe to changes
            let installed_ids: std::collections::HashSet<String> =
                PackageRegistry::load().into_iter().map(|p| p.id).collect();
            packages.write().iter_mut().for_each(|p| {
                p.installed = installed_ids.contains(&p.id);
            });
        });
    }

    {
        use_future(move || {
            let mut packages = packages;
            async move {
                let reader = StoreReader::official();
                match reader.load_all().await {
                    Ok(ns_map) => {
                        let inst_map = installed_map();
                        let entries: Vec<PackageEntry> = ns_map
                            .all()
                            .map(|pkg| {
                                let node_pkg = NodePackage::from_package(pkg);
                                PackageEntry::from_node_package(node_pkg, &inst_map)
                            })
                            .collect();
                        packages.set(entries);
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load catalog: {e}")));
                        packages.set(Vec::new());
                    }
                }
                loading.set(false);
            }
        });
    }

    let query = search.to_lowercase();
    // Split query into individual words — all must match (AND logic)
    let query_words: Vec<String> = query
        .split_whitespace()
        .map(std::string::ToString::to_string)
        .collect();

    let cur_filter = install_filter.read().clone();
    let filtered: Vec<PackageEntry> = packages
        .read()
        .iter()
        .filter(|p| {
            let matches_search = query_words.is_empty()
                || query_words.iter().all(|word| {
                    p.name.to_lowercase().contains(word.as_str())
                        || p.description.to_lowercase().contains(word.as_str())
                        || p.category.to_lowercase().contains(word.as_str())
                        || p.tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(word.as_str()))
                });
            let matches_kind = kinds.is_empty() || kinds.contains(&p.kind);
            let matches_install = match &cur_filter {
                InstallFilter::All => true,
                InstallFilter::Installed => p.installed,
                InstallFilter::Available => !p.installed,
                InstallFilter::Updatable => p.update_available,
            };
            matches_search && matches_kind && matches_install
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "fs-browser",
            // ── Install filter bar ──────────────────────────────────────────────
            div {
                style: "display: flex; gap: 6px; margin-bottom: 12px;",
                for (label, variant) in [
                    (fs_i18n::t("store.filter.all"),       InstallFilter::All),
                    (fs_i18n::t("store.filter.installed"), InstallFilter::Installed),
                    (fs_i18n::t("store.filter.available"), InstallFilter::Available),
                    (fs_i18n::t("store.filter.updatable"), InstallFilter::Updatable),
                ] {
                    {
                        let active = *install_filter.read() == variant;
                        let v = variant.clone();
                        rsx! {
                            button {
                                key: "{label}",
                                style: if active {
                                    "padding: 4px 12px; font-size: 12px; border-radius: var(--fs-radius-sm); \
                                     border: 1px solid var(--fs-color-primary); cursor: pointer; \
                                     background: var(--fs-color-primary); color: white;"
                                } else {
                                    "padding: 4px 12px; font-size: 12px; border-radius: var(--fs-radius-sm); \
                                     border: 1px solid var(--fs-color-border-default); cursor: pointer; \
                                     background: transparent; color: var(--fs-color-text-primary);"
                                },
                                onclick: move |_| install_filter.set(v.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }

            if *loading.read() {
                LoadingOverlay {
                    size: SpinnerSize::Lg,
                    message: Some(fs_i18n::t("store.loading_catalog").into()),
                }
            } else if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fs-color-error); background: rgba(239,68,68,0.1); \
                            border: 1px solid var(--fs-color-error); border-radius: 6px; \
                            padding: 12px; font-size: 13px;",
                    p { strong { {fs_i18n::t("store.unavailable")} } }
                    p { "{err}" }
                    p { style: "color: var(--fs-color-text-muted); font-size: 12px;",
                        {fs_i18n::t("store.offline_hint")}
                    }
                }
            } else if filtered.is_empty() {
                div {
                    style: "text-align: center; color: var(--fs-color-text-muted); padding: 48px;",
                    p { {fs_i18n::t_with("store.no_match", &[("search", search.as_str())])} }
                }
            } else {
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px;",
                    for pkg in filtered {
                        {
                            let pkg_for_install = pkg.clone();
                            let pkg_id_remove   = pkg.id.clone();
                            let is_bundle       = pkg.kind == PackageKind::Bundle;
                            rsx! {
                                PackageCard {
                                    key: "{pkg.id}",
                                    package: pkg.clone(),
                                    on_details: {
                                        let p = pkg.clone();
                                        move |_| on_select.call(p.clone())
                                    },
                                    on_install: Some(EventHandler::new(move |_| {
                                        let pkg2 = pkg_for_install.clone();
                                        spawn(async move {
                                            let _ = do_install(pkg2, String::new()).await;
                                            notify_install_changed();
                                        });
                                    })),
                                    on_remove: Some(EventHandler::new(move |_| {
                                        if is_bundle {
                                            let _ = PackageRegistry::remove_bundle(&pkg_id_remove);
                                        } else {
                                            let _ = PackageRegistry::remove(&pkg_id_remove);
                                        }
                                        notify_install_changed();
                                    })),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl PackageEntry {
    /// Build a `PackageEntry` from a `NodePackage`.
    ///
    /// `installed_map` maps package IDs to their `installed_by` bundle ID (if any).
    pub(crate) fn from_node_package(
        pkg: NodePackage,
        installed_map: &HashMap<String, Option<String>>,
    ) -> Self {
        let installed = installed_map.contains_key(&pkg.id);
        let installed_by = installed_map
            .get(&pkg.id)
            .and_then(std::clone::Clone::clone);
        let icon = pkg.icon.and_then(|i| resolve_icon_path(&i));
        PackageEntry {
            id: pkg.id,
            name: pkg.name,
            description: pkg.description,
            version: pkg.version,
            category: pkg.category,
            kind: pkg.kind,
            capabilities: pkg.capabilities,
            tags: pkg.tags,
            icon,
            store_path: pkg.path,
            installed,
            update_available: false,
            license: pkg.license,
            author: pkg.author,
            installed_by,
        }
    }
}

/// Build the installed-package map once per fetch cycle.
fn installed_map() -> HashMap<String, Option<String>> {
    PackageRegistry::load()
        .into_iter()
        .map(|p| (p.id, p.installed_by))
        .collect()
}
