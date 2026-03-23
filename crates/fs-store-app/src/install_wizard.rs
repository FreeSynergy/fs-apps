/// Install logic and result popup — replaces the multi-step install wizard.
use dioxus::prelude::*;
use fs_db_desktop::package_registry::{InstalledPackage, PackageRegistry};
use fs_store::StoreClient;

use crate::browser::resolve_icon;
use crate::node_package::{NodePackage, PackageKind};
use crate::package_card::PackageEntry;

// ── InstallResult ──────────────────────────────────────────────────────────────

/// Result of an install operation.
#[derive(Clone, PartialEq, Debug)]
pub enum InstallResult {
    Success,
    Failed(String),
}

// ── Install strategy ──────────────────────────────────────────────────────────

/// Context passed to each install strategy.
struct InstallCtx<'a> {
    fs_dir:       &'a std::path::Path,
    store_path:   Option<String>,
    env_vars:     &'a str,
    installed_by: Option<&'a str>,
}

/// Extension trait — each PackageKind knows how to download/install its files.
trait PackageInstallExt {
    async fn install_files(
        &self,
        package: &PackageEntry,
        ctx:     InstallCtx<'_>,
    ) -> Result<Option<String>, String>;
}

impl PackageInstallExt for PackageKind {
    async fn install_files(
        &self,
        package: &PackageEntry,
        ctx:     InstallCtx<'_>,
    ) -> Result<Option<String>, String> {
        match self {
            PackageKind::Bundle    => { install_bundle_members(package, ctx.fs_dir).await?; Ok(None) }
            PackageKind::Container => install_container(package, ctx.store_path, ctx.fs_dir, ctx.env_vars).await,
            PackageKind::Language  => install_language_pack(package, ctx.store_path, ctx.fs_dir).await,
            PackageKind::Theme     => install_theme_file(package, ctx.store_path, ctx.fs_dir).await,
            PackageKind::App       => install_app_binary(package, ctx.installed_by).await,
            // Widget, Bot, Task, Bridge, Plugin — register without file download
            _                      => Ok(None),
        }
    }
}

// ── async install logic ────────────────────────────────────────────────────────

/// Fetch an icon: if `icon` is an HTTP(S) URL, download and return the SVG content.
/// Falls back to the original value on network errors so installs never fail over icons.
async fn fetch_icon_content(icon: Option<String>) -> String {
    match icon {
        None => String::new(),
        Some(url) if url.starts_with("http://") || url.starts_with("https://") => {
            match reqwest::Client::new()
                .get(&url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    resp.text().await.unwrap_or(url)
                }
                _ => url,
            }
        }
        Some(other) => other,
    }
}

/// Downloads and registers a package. Returns Ok on success.
pub async fn do_install(package: PackageEntry, env_vars: String) -> Result<(), String> {
    do_install_inner(package, env_vars, None).await
}

/// Internal install: same as do_install but sets installed_by (for bundle members).
async fn do_install_inner(
    package:      PackageEntry,
    env_vars:     String,
    installed_by: Option<String>,
) -> Result<(), String> {
    let home      = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let fs_dir    = std::path::PathBuf::from(&home).join(".local/share/fsn");
    let store_path = package.store_path.as_deref()
        .map(|p| p.trim_end_matches('/').to_string());

    let ctx = InstallCtx {
        fs_dir:       &fs_dir,
        store_path,
        env_vars:     &env_vars,
        installed_by: installed_by.as_deref(),
    };
    // Fetch files and icon content concurrently.
    let (file_path, icon_content) = tokio::join!(
        package.kind.install_files(&package, ctx),
        fetch_icon_content(package.icon.clone()),
    );
    let file_path = file_path?;

    PackageRegistry::install(InstalledPackage {
        id:           package.id.clone(),
        name:         package.name.clone(),
        kind:         package.kind.clone(),
        version:      package.version.clone(),
        icon:         icon_content,
        file_path,
        installed_by,
        pinned: false,
    })
    .map_err(|e| format!("Registry error: {e}"))
}

/// Install all member packages of a bundle.
///
/// Fetches all catalogs, resolves each capability ID to a full PackageEntry,
/// and installs the member if not already installed. Members installed this
/// way are tagged with `installed_by = Some(bundle.id)`.
async fn install_bundle_members(bundle: &PackageEntry, _fs_dir: &std::path::Path) -> Result<(), String> {
    let pkg_map = fetch_catalog_map().await;

    for cap_id in &bundle.capabilities {
        // Skip if already installed (individually or via another bundle)
        if PackageRegistry::is_installed(cap_id) {
            tracing::debug!("bundle member '{cap_id}' already installed — skipping");
            continue;
        }

        match pkg_map.get(cap_id) {
            Some(member) => {
                tracing::info!("installing bundle member '{}' (via bundle '{}')", cap_id, bundle.id);
                Box::pin(do_install_inner(member.clone(), String::new(), Some(bundle.id.clone()))).await?;
            }
            None => {
                tracing::warn!("bundle member '{}' not found in any catalog — skipping", cap_id);
            }
        }
    }
    Ok(())
}

/// Fetch all catalogs (desktop, node, shared) and return a map of id → PackageEntry.
async fn fetch_catalog_map() -> std::collections::HashMap<String, PackageEntry> {
    let mut client = StoreClient::node_store();
    let mut map    = std::collections::HashMap::new();

    for namespace in &["apps", "desktop", "node", "shared"] {
        if let Ok(catalog) = client.fetch_catalog::<NodePackage>(namespace, false).await {
            for pkg in catalog.packages {
                let icon = pkg.icon.and_then(|i| resolve_icon(&i));
                map.insert(pkg.id.clone(), PackageEntry {
                    id:               pkg.id,
                    name:             pkg.name,
                    description:      pkg.description,
                    version:          pkg.version,
                    category:         pkg.category,
                    kind:             pkg.kind,
                    capabilities:     pkg.capabilities,
                    tags:             pkg.tags,
                    icon,
                    store_path:       pkg.path,
                    installed:        false,
                    update_available: false,
                    license:          pkg.license,
                    author:           pkg.author,
                    installed_by:     None,
                });
            }
        }
    }
    map
}

/// Install a binary app package.
///
/// Production: download from the distribution URL in the catalog (not yet implemented).
/// Dev mode (`FS_DEV=1`): use the locally compiled Cargo binary instead.
///
/// Dev binary resolution order:
///   1. `FS_BIN_{ID_UPPER}` env var — explicit override
///   2. `~/Server/FreeSynergy.{Title}/target/release/{binary}` — release build
///   3. `~/Server/FreeSynergy.{Title}/target/debug/{binary}` — debug build
///
/// `installed_by`: if Some, this is a bundle dependency — missing binaries are
/// skipped with a warning instead of failing the whole installation.
async fn install_app_binary(package: &PackageEntry, installed_by: Option<&str>) -> Result<Option<String>, String> {
    // Debug builds (cargo build / dx serve) are always dev mode.
    // FS_DEV=1 allows overriding in release builds (e.g. CI, staging).
    let is_dev = cfg!(debug_assertions)
        || std::env::var("FS_DEV").map(|v| v == "1").unwrap_or(false);
    if !is_dev {
        // Production: would download from catalog distribution URL.
        // Not yet implemented — just register without a file path for now.
        tracing::info!(
            "App '{}' registered (no binary download in production yet).",
            package.id
        );
        return Ok(None);
    }

    // Dev mode: find local Cargo build
    if let Some(path) = find_local_build_binary(&package.id) {
        let dest_dir = std::path::PathBuf::from(
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        ).join(".local/share/fsn/bin");
        std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

        let binary_name = std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| package.id.clone());
        let dest = dest_dir.join(&binary_name);

        std::fs::copy(&path, &dest).map_err(|e| {
            format!("Failed to copy local build '{}' to '{}': {e}", path, dest.display())
        })?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
        }

        tracing::info!(
            "[dev] App '{}' installed from local build: {} → {}",
            package.id, path, dest.display()
        );
        return Ok(Some(dest.to_string_lossy().into_owned()));
    }

    // Bundle dependency: skip gracefully so the rest of the bundle installs fine.
    if installed_by.is_some() {
        tracing::warn!(
            "[dev] App '{}' has no local build — skipping as bundle dependency. \
             Build it or set FS_BIN_{} to install it.",
            package.id,
            package.id.to_uppercase().replace('-', "_")
        );
        return Ok(None);
    }

    tracing::warn!(
        "[dev] No local build found for '{}' — registering without binary. \
         Build the project or set FS_BIN_{} to the binary path.",
        package.id,
        package.id.to_uppercase().replace('-', "_")
    );
    Ok(None)
}

/// Try to locate a locally compiled binary for a package.
///
/// Checks (in order):
///   1. `FS_BIN_{ID_UPPER}` env var
///   2. `~/Server/FreeSynergy.{Title}/target/release/{binary}`
///   3. `~/Server/FreeSynergy.{Title}/target/debug/{binary}`
fn find_local_build_binary(id: &str) -> Option<String> {
    let env_key = format!("FS_BIN_{}", id.to_uppercase().replace('-', "_"));
    if let Ok(path) = std::env::var(&env_key) {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base = std::path::PathBuf::from(&home).join("Server");

    // Map known catalog IDs (both "fs-*" prefixed and bare) → (repo title, binary name).
    // The catalog uses "fs-{name}" IDs; older internal code used bare IDs.
    let known: Option<(&str, &str)> = match id {
        "node"    | "fs-node"                          => Some(("Node",    "fsn")),
        "desktop" | "fs-desktop" | "store" | "fs-store" => Some(("Desktop", "fs-desktop")),
        "init"    | "fs-init"                          => Some(("Init",    "fs-init")),
        "browser" | "fs-browser"                       => Some(("Browser", "fs-browser")),
        _                                               => None,
    };

    let (title, binary_name): (String, String) = if let Some((t, b)) = known {
        (t.to_string(), b.to_string())
    } else {
        // Strip "fs-" prefix, capitalize remainder for the repo folder name.
        let stripped = id.strip_prefix("fs-").unwrap_or(id);
        let cap: String = {
            let mut c = stripped.chars();
            match c.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };
        (cap, id.to_string())
    };

    let repo = base.join(format!("FreeSynergy.{title}"));
    for profile in &["release", "debug"] {
        let path = repo.join("target").join(profile).join(&binary_name);
        if path.exists() {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    None
}

/// Download and install a language pack (`ui.toml`).
async fn install_language_pack(
    package:    &PackageEntry,
    store_path: Option<String>,
    fs_dir:     &std::path::Path,
) -> Result<Option<String>, String> {
    let base = store_path.unwrap_or_else(|| format!("shared/i18n/{}", package.id));
    let url  = format!("{base}/ui.toml");
    match StoreClient::node_store().fetch_raw(&url).await {
        Ok(content) => {
            let dest_dir = fs_dir.join("i18n").join(&package.id);
            std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            let dest = dest_dir.join("ui.toml");
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            Ok(Some(dest.to_string_lossy().into_owned()))
        }
        Err(e) => {
            tracing::warn!("Language pack download failed (registering anyway): {e}");
            Ok(None)
        }
    }
}

/// Download and install a theme file (`theme.css`).
async fn install_theme_file(
    package:    &PackageEntry,
    store_path: Option<String>,
    fs_dir:     &std::path::Path,
) -> Result<Option<String>, String> {
    let base = store_path.unwrap_or_else(|| format!("shared/themes/{}", package.id));
    let url  = format!("{base}/theme.css");
    match StoreClient::node_store().fetch_raw(&url).await {
        Ok(content) => {
            let dest_dir = fs_dir.join("themes");
            std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            let dest = dest_dir.join(format!("{}.css", package.id));
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            Ok(Some(dest.to_string_lossy().into_owned()))
        }
        Err(e) => {
            tracing::warn!("Theme download failed (registering anyway): {e}");
            Ok(None)
        }
    }
}

/// Full container install:
///  1. Fetch compose file from store
///  2. Write compose + .env to ~/.local/share/fsn/services/<id>/
///  3. Try `fsn container install <compose_path>` (adds Quadlet + systemd unit)
///  4. systemctl --user daemon-reload
async fn install_container(
    package:    &PackageEntry,
    store_path: Option<String>,
    fs_dir:    &std::path::Path,
    env_vars:   &str,
) -> Result<Option<String>, String> {
    let base = store_path.unwrap_or_else(|| format!("node/modules/{}", package.id));

    // Try compose.yml first, then docker-compose.yml
    let compose_content = {
        let mut content = None;
        for name in &["compose.yml", "docker-compose.yml", "container.yml"] {
            let url = format!("{base}/{name}");
            if let Ok(c) = StoreClient::node_store().fetch_raw(&url).await {
                content = Some((c, *name));
                break;
            }
        }
        content
    };

    let service_dir = fs_dir.join("services").join(&package.id);
    std::fs::create_dir_all(&service_dir).map_err(|e| e.to_string())?;

    let compose_path = match compose_content {
        Some((content, filename)) => {
            let dest = service_dir.join(filename);
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            dest
        }
        None => {
            // No compose file in store — create a placeholder so the user can
            // edit it manually and re-run `fsn container install`.
            let dest = service_dir.join("compose.yml");
            std::fs::write(&dest, format!(
                "# Compose file for {name}\n\
                 # Edit this file and run: fsn container install {path}\n\
                 services:\n\
                 #  {id}:\n\
                 #    image: ...\n",
                name = package.name,
                id   = package.id,
                path = dest.display(),
            )).map_err(|e| e.to_string())?;
            dest
        }
    };

    // Write .env file
    if !env_vars.trim().is_empty() {
        let env_path = service_dir.join(".env");
        std::fs::write(&env_path, env_vars).map_err(|e| e.to_string())?;
    }

    // Try `fsn container install <compose_path>`
    let compose_str = compose_path.to_string_lossy().into_owned();
    let container_result = tokio::process::Command::new("fsn")
        .args(["container", "install", &compose_str])
        .output()
        .await;

    match container_result {
        Ok(out) if out.status.success() => {
            // Reload systemd so the new Quadlet unit is picked up
            let _ = tokio::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
                .await;
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(
                "fsn container install returned non-zero: {stderr}. \
                 Compose file saved to {compose_str} — run manually to finish setup."
            );
        }
        Err(_) => {
            tracing::warn!(
                "`fsn` binary not found. Compose file saved to {compose_str}. \
                 Run `fsn container install {compose_str}` manually to activate the service."
            );
        }
    }

    Ok(Some(compose_str))
}

// ── Fetch detected env vars from compose file ─────────────────────────────────

/// Fetches a compose file from the store and extracts environment variable
/// names so the configure step can show pre-populated input fields.
pub async fn fetch_container_env_vars(package: &PackageEntry) -> Vec<String> {
    let base = package.store_path.as_deref()
        .map(|p| p.trim_end_matches('/').to_string())
        .unwrap_or_else(|| format!("node/modules/{}", package.id));

    for name in &["compose.yml", "docker-compose.yml", "container.yml"] {
        let url = format!("{base}/{name}");
        if let Ok(content) = StoreClient::node_store().fetch_raw(&url).await {
            return extract_env_var_names(&content);
        }
    }
    vec![]
}

/// Simple line-by-line extraction of `KEY` or `KEY: ...` or `KEY=...` from a
/// YAML environment section. Good enough for showing input fields; the
/// Conductor will do the proper analysis when installing.
pub fn extract_env_var_names(yaml: &str) -> Vec<String> {
    let mut in_env = false;
    let mut vars   = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim();

        // Detect `environment:` section start
        if trimmed == "environment:" || trimmed.starts_with("environment:") {
            in_env = true;
            continue;
        }

        // Detect next top-level key (ends env section)
        if in_env && !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
            in_env = false;
        }

        if in_env {
            // `  - KEY=value`  or  `  - KEY`
            let entry = trimmed.trim_start_matches("- ");
            // `  KEY: value`
            let key = if let Some(pos) = entry.find('=') {
                &entry[..pos]
            } else if let Some(pos) = entry.find(':') {
                &entry[..pos]
            } else {
                entry
            };
            let key = key.trim();
            if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                vars.push(key.to_string());
            }
        }
    }
    vars
}

// ── InstallPopup ───────────────────────────────────────────────────────────────

/// Simple centered modal overlay shown after an install attempt.
#[component]
pub fn InstallPopup(result: InstallResult, on_close: EventHandler<()>) -> Element {
    let (icon, title, detail) = match &result {
        InstallResult::Success => (
            "✅",
            fs_i18n::t("store.install.success"),
            String::new(),
        ),
        InstallResult::Failed(err) => (
            "❌",
            fs_i18n::t("store.install.failed"),
            err.clone(),
        ),
    };

    rsx! {
        // Overlay backdrop
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.55); \
                    display: flex; align-items: center; justify-content: center; z-index: 2000;",
            onclick: move |_| on_close.call(()),

            // Modal card — stop propagation so clicking inside doesn't close
            div {
                style: "background: var(--fs-color-bg-surface); \
                        border: 1px solid var(--fs-color-border-default); \
                        border-radius: var(--fs-radius-lg); \
                        padding: 40px 32px; max-width: 360px; width: 100%; \
                        text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.4);",
                onclick: move |e| e.stop_propagation(),

                p { style: "font-size: 48px; margin: 0 0 16px 0;", "{icon}" }
                p { style: "font-size: 18px; font-weight: 600; margin: 0 0 12px 0;",
                    "{title}"
                }
                if !detail.is_empty() {
                    p {
                        style: "font-size: 13px; color: var(--fs-color-error, #ef4444); \
                                background: rgba(239,68,68,0.08); \
                                border: 1px solid var(--fs-color-error, #ef4444); \
                                border-radius: var(--fs-radius-md); \
                                padding: 10px 12px; margin: 0 0 20px 0; \
                                text-align: left; word-break: break-word;",
                        "{detail}"
                    }
                }
                button {
                    style: "padding: 10px 32px; background: var(--fs-color-primary); \
                            color: white; border: none; \
                            border-radius: var(--fs-radius-md); cursor: pointer; \
                            font-size: 14px; font-weight: 600;",
                    onclick: move |_| on_close.call(()),
                    {fs_i18n::t("actions.close")}
                }
            }
        }
    }
}
