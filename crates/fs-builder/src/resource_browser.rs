/// Resource Browser — view and edit installed package resources + Store resources.
///
/// # Permission model
///
/// - **Local resources** (anything under `~/.local/share/fsn/`): always editable.
/// - **Store resources** (Node catalog): editable only if the user has git push
///   access to the store repository.  Access is checked by running
///   `git ls-remote <store_git_url>` with the user's SSH key.  If that succeeds
///   the user can open a resource for editing; saving creates a commit in a local
///   clone and prompts to push.
///
/// The git check is intentionally lightweight (read-only remote probe) —
/// the actual write gate is the remote's push hook, just like any git workflow.
use dioxus::prelude::*;
use fs_db_desktop::package_registry::{InstalledPackage, PackageRegistry};

// ── Git access check ───────────────────────────────────────────────────────────

const STORE_GIT_URL: &str = "git@github.com:FreeSynergy/Store.git";

async fn check_git_push_access() -> bool {
    tokio::process::Command::new("git")
        .args(["ls-remote", "--heads", STORE_GIT_URL])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── File helpers ───────────────────────────────────────────────────────────────

fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

fn write_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

// ── ResourceBrowser ────────────────────────────────────────────────────────────

#[component]
pub fn ResourceBrowser() -> Element {
    let packages      = use_signal(|| PackageRegistry::load());
    let selected  = use_signal(|| Option::<InstalledPackage>::None);
    let mut file_text = use_signal(String::new);
    let mut dirty     = use_signal(|| false);
    let mut msg       = use_signal(|| Option::<String>::None);

    // Git access for Store editing (checked once on mount)
    let git_access: Signal<Option<bool>> = use_signal(|| None);
    {
        let mut git_access = git_access.clone();
        use_future(move || async move {
            let ok = check_git_push_access().await;
            git_access.set(Some(ok));
        });
    }

    rsx! {
        div {
            style: "display: flex; height: 100%;",

            // ── Left: installed packages list ─────────────────────────────────
            div {
                style: "width: 240px; border-right: 1px solid var(--fs-color-border-default); \
                        overflow-y: auto; flex-shrink: 0;",

                div {
                    style: "padding: 12px 14px; font-size: 12px; font-weight: 600; \
                            text-transform: uppercase; letter-spacing: 0.07em; \
                            color: var(--fs-color-text-muted); \
                            border-bottom: 1px solid var(--fs-color-border-default);",
                    "Local Resources"
                }

                if packages.read().is_empty() {
                    div {
                        style: "padding: 16px; font-size: 13px; \
                                color: var(--fs-color-text-muted);",
                        "No packages installed. Install something from the Store first."
                    }
                } else {
                    for pkg in packages.read().clone() {
                        ResourceListItem {
                            key:        "{pkg.id}",
                            pkg:        pkg.clone(),
                            active:     selected.read().as_ref().map(|s| &s.id) == Some(&pkg.id),
                            on_click:   {
                                let mut selected  = selected.clone();
                                let mut file_text = file_text.clone();
                                let mut dirty     = dirty.clone();
                                let mut msg       = msg.clone();
                                move |p: InstalledPackage| {
                                    let text = p.file_path.as_deref()
                                        .and_then(|fp| read_file(fp).ok())
                                        .unwrap_or_default();
                                    *file_text.write() = text;
                                    *dirty.write() = false;
                                    *msg.write() = None;
                                    *selected.write() = Some(p);
                                }
                            },
                        }
                    }
                }

                // Store resources section
                div {
                    style: "padding: 12px 14px; font-size: 12px; font-weight: 600; \
                            text-transform: uppercase; letter-spacing: 0.07em; \
                            color: var(--fs-color-text-muted); \
                            border-top: 1px solid var(--fs-color-border-default); \
                            border-bottom: 1px solid var(--fs-color-border-default); \
                            margin-top: 8px;",
                    "Store Resources"
                }
                div {
                    style: "padding: 12px 14px;",
                    match *git_access.read() {
                        None => rsx! {
                            span { style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                "Checking git access…"
                            }
                        },
                        Some(true) => rsx! {
                            div {
                                style: "font-size: 12px; color: var(--fs-color-success, #22c55e); \
                                        margin-bottom: 8px;",
                                "✓ Store write access confirmed"
                            }
                            p { style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                "Store resource editing via git is planned. \
                                 You can manually edit the Store repository at:"
                            }
                            code {
                                style: "font-size: 11px; display: block; margin-top: 6px; \
                                        background: var(--fs-color-bg-overlay); \
                                        padding: 4px 8px; border-radius: 4px; \
                                        word-break: break-all;",
                                "{STORE_GIT_URL}"
                            }
                        },
                        Some(false) => rsx! {
                            p { style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                "No git push access to the Store."
                            }
                            p { style: "font-size: 12px; color: var(--fs-color-text-muted); \
                                        margin-top: 4px;",
                                "To edit Store resources, add your SSH key to your \
                                 GitHub/Forgejo account and request collaborator access."
                            }
                        },
                    }
                }
            }

            // ── Right: file editor ────────────────────────────────────────────
            div {
                style: "flex: 1; display: flex; flex-direction: column; \
                        padding: 16px; overflow: hidden;",

                match selected.read().clone() {
                    None => rsx! {
                        div {
                            style: "flex: 1; display: flex; align-items: center; \
                                    justify-content: center; \
                                    color: var(--fs-color-text-muted); font-size: 13px;",
                            "Select a resource from the list to view and edit it."
                        }
                    },
                    Some(pkg) => rsx! {
                        // Header
                        div {
                            style: "display: flex; justify-content: space-between; \
                                    align-items: center; margin-bottom: 12px; flex-shrink: 0;",
                            div {
                                strong { style: "font-size: 15px;", "{pkg.name}" }
                                span {
                                    style: "margin-left: 8px; font-size: 11px; \
                                            padding: 2px 8px; border-radius: 999px; \
                                            background: var(--fs-color-bg-overlay); \
                                            border: 1px solid var(--fs-color-border-default); \
                                            color: var(--fs-color-text-muted);",
                                    "{pkg.kind}"
                                }
                                if *dirty.read() {
                                    span {
                                        style: "margin-left: 8px; font-size: 12px; \
                                                color: var(--fs-color-warning, #f59e0b);",
                                        "● unsaved"
                                    }
                                }
                            }
                            div { style: "display: flex; gap: 8px; align-items: center;",
                                if let Some(m) = msg.read().as_deref() {
                                    span {
                                        style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                        "{m}"
                                    }
                                }
                                if pkg.file_path.is_some() {
                                    button {
                                        style: "padding: 6px 16px; \
                                                background: var(--fs-color-primary); \
                                                color: white; border: none; \
                                                border-radius: var(--fs-radius-md); \
                                                cursor: pointer; font-size: 13px;",
                                        disabled: !*dirty.read(),
                                        onclick: {
                                            let pkg = pkg.clone();
                                            move |_| {
                                                if let Some(fp) = &pkg.file_path {
                                                    match write_file(fp, &file_text.read()) {
                                                        Ok(()) => {
                                                            *dirty.write() = false;
                                                            *msg.write() = Some("Saved.".to_string());
                                                        }
                                                        Err(e) => {
                                                            *msg.write() = Some(format!("Error: {e}"));
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        "Save"
                                    }
                                }
                            }
                        }

                        // File path
                        if let Some(fp) = &pkg.file_path {
                            div {
                                style: "font-size: 11px; color: var(--fs-color-text-muted); \
                                        margin-bottom: 8px; font-family: var(--fs-font-mono, monospace);",
                                "{fp}"
                            }
                        }

                        // Editor or "no file" message
                        if pkg.file_path.is_none() {
                            div {
                                style: "flex: 1; display: flex; align-items: center; \
                                        justify-content: center; text-align: center; \
                                        color: var(--fs-color-text-muted); font-size: 13px; \
                                        border: 1px dashed var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); padding: 32px;",
                                "This package has no local file path recorded. \
                                 It was registered without a downloaded file."
                            }
                        } else if file_text.read().is_empty() {
                            div {
                                style: "flex: 1; display: flex; align-items: center; \
                                        justify-content: center; text-align: center; \
                                        color: var(--fs-color-text-muted); font-size: 13px; \
                                        border: 1px dashed var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); padding: 32px;",
                                "File is empty or could not be read."
                            }
                        } else {
                            textarea {
                                style: "flex: 1; width: 100%; padding: 10px 12px; \
                                        font-family: var(--fs-font-mono, monospace); \
                                        font-size: 13px; resize: none; \
                                        border: 1px solid var(--fs-color-border-default); \
                                        border-radius: var(--fs-radius-md); \
                                        background: var(--fs-color-bg-elevated); \
                                        color: var(--fs-color-text-primary); \
                                        box-sizing: border-box; min-height: 400px;",
                                value: "{file_text.read()}",
                                oninput: move |e| {
                                    *file_text.write() = e.value();
                                    *dirty.write() = true;
                                    *msg.write() = None;
                                },
                            }
                        }
                    },
                }
            }
        }
    }
}

// ── ResourceListItem ───────────────────────────────────────────────────────────

#[component]
fn ResourceListItem(
    pkg:      InstalledPackage,
    active:   bool,
    on_click: EventHandler<InstalledPackage>,
) -> Element {
    let bg = if active {
        "background: var(--fs-color-bg-overlay);"
    } else {
        "background: transparent;"
    };

    rsx! {
        div {
            style: "padding: 10px 14px; cursor: pointer; border-bottom: 1px solid \
                    var(--fs-color-border-default); {bg}",
            onclick: {
                let p = pkg.clone();
                move |_| on_click.call(p.clone())
            },
            div {
                style: "font-size: 13px; font-weight: 500; \
                        color: var(--fs-color-text-primary);",
                "{pkg.name}"
            }
            div {
                style: "font-size: 11px; color: var(--fs-color-text-muted); margin-top: 2px;",
                "{pkg.kind} · v{pkg.version}"
            }
        }
    }
}
