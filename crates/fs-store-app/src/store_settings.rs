/// Store Settings — manage configured package repositories.
///
/// The primary/official store can only be toggled (enabled/disabled), not edited or deleted.
/// All other stores support full CRUD.
use dioxus::prelude::*;
use fs_i18n;
use serde::{Deserialize, Serialize};

// ── Data types ─────────────────────────────────────────────────────────────────

/// A single package store entry (mirrors StoreConfig in FreeSynergy.Node).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RepoEntry {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub git_url: Option<String>,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default = "bool_true")]
    pub enabled: bool,
    /// Built-in official store — can only be toggled, not edited or deleted.
    #[serde(default)]
    pub primary: bool,
}

fn bool_true() -> bool { true }

/// The one official FreeSynergy Store — always present, always primary.
pub fn official_store() -> RepoEntry {
    RepoEntry {
        name:       "FreeSynergy Store".to_string(),
        url:        "https://raw.githubusercontent.com/FreeSynergy/Store/main".to_string(),
        git_url:    Some("https://github.com/FreeSynergy/Store".to_string()),
        local_path: None,
        enabled:    true,
        primary:    true,
    }
}

fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".config").join("fsn").join("settings.toml")
}

// ── Persistence ────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct PartialSettings {
    #[serde(default)]
    stores: Vec<RepoEntry>,
}

pub fn load_repos() -> Vec<RepoEntry> {
    let content = std::fs::read_to_string(config_path()).unwrap_or_default();
    let parsed: PartialSettings = toml::from_str(&content).unwrap_or_default();
    let mut stores = parsed.stores;

    // Always ensure the official store is present at index 0 as primary.
    let official = official_store();
    let already_present = stores.iter().any(|r| r.primary || r.url == official.url);
    if !already_present {
        stores.insert(0, official.clone());
    }

    // Force primary = true on any entry that matches the official URL.
    // Survives serialization round-trips that might have cleared the flag.
    for r in stores.iter_mut() {
        if r.url == official.url {
            r.primary = true;
        }
    }

    stores
}

pub fn save_repos(repos: &[RepoEntry]) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc: toml::Value = toml::from_str(&existing)
        .unwrap_or(toml::Value::Table(Default::default()));

    if let toml::Value::Table(ref mut root) = doc {
        let stores_val = toml::Value::try_from(repos.to_vec()).map_err(|e| e.to_string())?;
        root.insert("stores".to_string(), stores_val);
    }

    let out = toml::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&path, out).map_err(|e| e.to_string())
}

// ── Edit state ─────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum FormMode {
    Hidden,
    Editing(usize),
    Adding,
}

// ── Root component ─────────────────────────────────────────────────────────────

#[component]
pub fn StoreSettings() -> Element {
    let mut repos    = use_signal(load_repos);
    let mut mode     = use_signal(|| FormMode::Hidden);
    let mut form_buf = use_signal(RepoEntry::default);
    let mut save_msg = use_signal(|| Option::<String>::None);

    let items = repos.read().clone();

    rsx! {
        div {
            style: "padding: 24px;",

            // ── Header ──────────────────────────────────────────────────────────
            div {
                style: "display: flex; align-items: flex-start; justify-content: space-between; \
                        margin-bottom: 24px;",
                div {
                    h3 { style: "margin: 0;", {fs_i18n::t("store.settings.title")} }
                    p {
                        style: "margin: 4px 0 0; color: var(--fs-color-text-muted); font-size: 13px;",
                        {fs_i18n::t("store.settings.description")}
                    }
                }
                button {
                    style: "padding: 7px 16px; background: var(--fs-color-primary); color: white; \
                            border: none; border-radius: var(--fs-radius-md); cursor: pointer; \
                            font-size: 13px; white-space: nowrap; flex-shrink: 0;",
                    disabled: *mode.read() != FormMode::Hidden,
                    onclick: move |_| {
                        form_buf.set(RepoEntry::default());
                        mode.set(FormMode::Adding);
                    },
                    {fs_i18n::t("store.settings.btn_add")}
                }
            }

            // ── Repository list ─────────────────────────────────────────────────
            div {
                style: "display: flex; flex-direction: column; gap: 8px;",

                if items.is_empty() && *mode.read() == FormMode::Hidden {
                    div {
                        style: "padding: 32px; text-align: center; color: var(--fs-color-text-muted); \
                                background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); \
                                border: 1px dashed var(--fs-color-border-default);",
                        {fs_i18n::t("store.settings.empty")}
                    }
                }

                for (idx, entry) in items.iter().enumerate() {
                    {
                        let is_editing = *mode.read() == FormMode::Editing(idx);
                        if is_editing {
                            rsx! {
                                RepoForm {
                                    key: "form-{idx}",
                                    entry: form_buf.read().clone(),
                                    on_change: move |e| form_buf.set(e),
                                    on_save: move |_| {
                                        repos.write()[idx] = form_buf.read().clone();
                                        mode.set(FormMode::Hidden);
                                    },
                                    on_cancel: move |_| mode.set(FormMode::Hidden),
                                }
                            }
                        } else {
                            rsx! {
                                RepoRow {
                                    key: "row-{idx}",
                                    entry: entry.clone(),
                                    on_toggle: move |_| {
                                        let cur = repos.read()[idx].enabled;
                                        repos.write()[idx].enabled = !cur;
                                    },
                                    on_edit: move |_| {
                                        form_buf.set(repos.read()[idx].clone());
                                        mode.set(FormMode::Editing(idx));
                                    },
                                    on_delete: move |_| {
                                        repos.write().remove(idx);
                                    },
                                }
                            }
                        }
                    }
                }

                // ── Add form (appended at bottom) ───────────────────────────────
                if *mode.read() == FormMode::Adding {
                    RepoForm {
                        key: "form-new",
                        entry: form_buf.read().clone(),
                        on_change: move |e| form_buf.set(e),
                        on_save: move |_| {
                            repos.write().push(form_buf.read().clone());
                            mode.set(FormMode::Hidden);
                        },
                        on_cancel: move |_| mode.set(FormMode::Hidden),
                    }
                }
            }

            // ── Save ─────────────────────────────────────────────────────────────
            div { style: "margin-top: 24px; display: flex; align-items: center; gap: 12px;",
                button {
                    style: "padding: 8px 24px; background: var(--fs-color-primary); color: white; \
                            border: none; border-radius: var(--fs-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        match save_repos(&repos.read()) {
                            Ok(())  => *save_msg.write() = Some(fs_i18n::t("notifications.saved").to_string()),
                            Err(e)  => *save_msg.write() = Some(format!("Error: {e}")),
                        }
                    },
                    {fs_i18n::t("actions.save")}
                }
                if let Some(msg) = save_msg.read().as_deref() {
                    span { style: "font-size: 13px; color: var(--fs-color-text-muted);", "{msg}" }
                }
            }
        }
    }
}

// ── RepoRow ────────────────────────────────────────────────────────────────────

#[component]
fn RepoRow(
    entry: RepoEntry,
    on_toggle: EventHandler<()>,
    on_edit: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let dot_color = if entry.enabled {
        "var(--fs-color-success, #22c55e)"
    } else {
        "var(--fs-color-text-muted)"
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; padding: 12px 16px; \
                    background: var(--fs-color-bg-surface); border-radius: var(--fs-radius-md); \
                    border: 1px solid var(--fs-color-border-default);",

            div {
                style: "width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; \
                        background: {dot_color};",
            }

            div { style: "flex: 1; min-width: 0;",
                div { style: "display: flex; align-items: center; gap: 8px; flex-wrap: wrap;",
                    strong { style: "font-size: 14px;", "{entry.name}" }
                    if entry.primary {
                        span {
                            style: "font-size: 11px; background: var(--fs-color-primary); color: white; \
                                    padding: 1px 6px; border-radius: 4px;",
                            {fs_i18n::t("store.settings.badge_primary")}
                        }
                    }
                    if !entry.enabled {
                        span {
                            style: "font-size: 11px; color: var(--fs-color-text-muted);",
                            {fs_i18n::t("store.settings.badge_disabled")}
                        }
                    }
                }
                div {
                    style: "font-size: 12px; color: var(--fs-color-text-muted); margin-top: 2px; \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{entry.url}"
                }
            }

            div { style: "display: flex; align-items: center; gap: 8px; flex-shrink: 0;",
                button {
                    style: "padding: 5px 12px; border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); cursor: pointer; font-size: 12px; \
                            background: var(--fs-color-bg-base); color: var(--fs-color-text-primary);",
                    onclick: move |_| on_toggle.call(()),
                    if entry.enabled {
                        {fs_i18n::t("actions.disable")}
                    } else {
                        {fs_i18n::t("actions.enable")}
                    }
                }
                if !entry.primary {
                    button {
                        style: "padding: 5px 12px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); cursor: pointer; font-size: 12px; \
                                background: var(--fs-color-bg-base); color: var(--fs-color-text-primary);",
                        onclick: move |_| on_edit.call(()),
                        {fs_i18n::t("actions.edit")}
                    }
                    button {
                        style: "padding: 5px 12px; border: 1px solid var(--fs-color-error, #ef4444); \
                                border-radius: var(--fs-radius-md); cursor: pointer; font-size: 12px; \
                                color: var(--fs-color-error, #ef4444); background: transparent;",
                        onclick: move |_| on_delete.call(()),
                        {fs_i18n::t("actions.delete")}
                    }
                }
            }
        }
    }
}

// ── RepoForm ───────────────────────────────────────────────────────────────────

#[component]
fn RepoForm(
    entry: RepoEntry,
    on_change: EventHandler<RepoEntry>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "padding: 16px; background: var(--fs-color-bg-surface); \
                    border-radius: var(--fs-radius-md); \
                    border: 1px solid var(--fs-color-primary);",

            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",

                div {
                    label {
                        style: "display: block; font-size: 12px; color: var(--fs-color-text-muted); margin-bottom: 4px;",
                        {fs_i18n::t("store.settings.label_name")}
                    }
                    input {
                        style: "width: 100%; padding: 7px 10px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px; \
                                background: var(--fs-color-bg-base); color: var(--fs-color-text-primary); \
                                box-sizing: border-box;",
                        value: "{entry.name}",
                        placeholder: "My Repository",
                        oninput: {
                            let entry = entry.clone();
                            move |e: Event<FormData>| {
                                on_change.call(RepoEntry { name: e.value(), ..entry.clone() });
                            }
                        },
                    }
                }

                div {
                    label {
                        style: "display: block; font-size: 12px; color: var(--fs-color-text-muted); margin-bottom: 4px;",
                        {fs_i18n::t("store.settings.label_url")}
                    }
                    input {
                        style: "width: 100%; padding: 7px 10px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px; \
                                background: var(--fs-color-bg-base); color: var(--fs-color-text-primary); \
                                box-sizing: border-box;",
                        value: "{entry.url}",
                        placeholder: "https://raw.githubusercontent.com/…",
                        oninput: {
                            let entry = entry.clone();
                            move |e: Event<FormData>| {
                                on_change.call(RepoEntry { url: e.value(), ..entry.clone() });
                            }
                        },
                    }
                }

                div {
                    label {
                        style: "display: block; font-size: 12px; color: var(--fs-color-text-muted); margin-bottom: 4px;",
                        {fs_i18n::t("store.settings.label_git_url")}
                    }
                    input {
                        style: "width: 100%; padding: 7px 10px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px; \
                                background: var(--fs-color-bg-base); color: var(--fs-color-text-primary); \
                                box-sizing: border-box;",
                        value: entry.git_url.as_deref().unwrap_or(""),
                        placeholder: "https://github.com/…  (optional)",
                        oninput: {
                            let entry = entry.clone();
                            move |e: Event<FormData>| {
                                let val = e.value();
                                let git_url = if val.is_empty() { None } else { Some(val) };
                                on_change.call(RepoEntry { git_url, ..entry.clone() });
                            }
                        },
                    }
                }

                div {
                    label {
                        style: "display: block; font-size: 12px; color: var(--fs-color-text-muted); margin-bottom: 4px;",
                        {fs_i18n::t("store.settings.label_local_path")}
                    }
                    input {
                        style: "width: 100%; padding: 7px 10px; border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); font-size: 13px; \
                                background: var(--fs-color-bg-base); color: var(--fs-color-text-primary); \
                                box-sizing: border-box;",
                        value: entry.local_path.as_deref().unwrap_or(""),
                        placeholder: "/home/user/FreeSynergy.Store  (optional)",
                        oninput: {
                            let entry = entry.clone();
                            move |e: Event<FormData>| {
                                let val = e.value();
                                let local_path = if val.is_empty() { None } else { Some(val) };
                                on_change.call(RepoEntry { local_path, ..entry.clone() });
                            }
                        },
                    }
                }
            }

            div { style: "display: flex; gap: 8px; margin-top: 16px; justify-content: flex-end;",
                button {
                    style: "padding: 7px 20px; border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px; \
                            background: transparent; color: var(--fs-color-text-primary);",
                    onclick: move |_| on_cancel.call(()),
                    {fs_i18n::t("actions.cancel")}
                }
                button {
                    style: "padding: 7px 20px; background: var(--fs-color-primary); color: white; \
                            border: none; border-radius: var(--fs-radius-md); cursor: pointer; font-size: 13px;",
                    disabled: entry.name.is_empty() || entry.url.is_empty(),
                    onclick: move |_| on_save.call(()),
                    {fs_i18n::t("actions.apply")}
                }
            }
        }
    }
}
