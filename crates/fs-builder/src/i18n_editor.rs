/// i18n editor — view and edit installed FSN language files (ui.toml).
use dioxus::prelude::*;
use fs_i18n;
use fs_db_desktop::package_registry::{PackageKind, PackageRegistry};

/// Built-in language codes (always shown, even without a registry entry).
const BUILTIN_LANGUAGES: &[(&str, &str)] = &[
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
];

// ── LangOption ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
struct LangOption {
    code: String,
    name: String,
}

fn load_lang_options() -> Vec<LangOption> {
    // Only show builtin languages that actually have a ui.toml installed on disk.
    let mut opts: Vec<LangOption> = BUILTIN_LANGUAGES
        .iter()
        .filter(|(code, _)| ui_toml_path(code).exists())
        .map(|(code, name)| LangOption { code: code.to_string(), name: name.to_string() })
        .collect();

    let builtin_codes: Vec<&str> = BUILTIN_LANGUAGES.iter().map(|(c, _)| *c).collect();
    for pkg in PackageRegistry::by_kind(PackageKind::Language) {
        if !builtin_codes.contains(&pkg.id.as_str()) {
            opts.push(LangOption { code: pkg.id, name: pkg.name });
        }
    }
    opts
}

// ── ui.toml path ───────────────────────────────────────────────────────────────

fn ui_toml_path(code: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".local/share/fsn/i18n")
        .join(code)
        .join("ui.toml")
}

fn load_ui_toml(code: &str) -> String {
    std::fs::read_to_string(ui_toml_path(code)).unwrap_or_default()
}

fn save_ui_toml(code: &str, content: &str) -> Result<(), String> {
    let path = ui_toml_path(code);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

// ── I18nEditor ────────────────────────────────────────────────────────────────

/// i18n editor component — shows installed languages and allows editing their ui.toml.
#[component]
pub fn I18nEditor() -> Element {
    let lang_options    = use_signal(load_lang_options);
    let mut selected    = use_signal(|| {
        load_lang_options().first().map(|l| l.code.clone()).unwrap_or_else(|| "de".to_string())
    });
    let mut content     = use_signal(|| load_ui_toml(&selected.read()));
    let mut save_msg    = use_signal(|| Option::<String>::None);
    let mut dirty       = use_signal(|| false);

    rsx! {
        div {
            class: "fs-i18n-editor",
            style: "display: flex; height: 100%;",

            // ── Sidebar ───────────────────────────────────────────────────────
            div {
                style: "width: 200px; border-right: 1px solid var(--fs-color-border-default); \
                        padding: 16px; display: flex; flex-direction: column; gap: 12px;",

                div {
                    label {
                        style: "display: block; font-size: 12px; font-weight: 600; \
                                margin-bottom: 8px; color: var(--fs-color-text-muted);",
                        {fs_i18n::t("builder.i18n.language_label")}
                    }
                    if lang_options.read().is_empty() {
                        p {
                            style: "font-size: 12px; color: var(--fs-color-text-muted);",
                            {fs_i18n::t("builder.i18n.no_packs")}
                        }
                    } else {
                        div {
                            style: "border: 1px solid var(--fs-color-border-default); \
                                    border-radius: var(--fs-radius-md); overflow: hidden;",
                            for opt in lang_options.read().clone() {
                                LangBtn {
                                    key:      "{opt.code}",
                                    code:     opt.code.clone(),
                                    name:     opt.name.clone(),
                                    active:   *selected.read() == opt.code,
                                    on_click: {
                                        let code = opt.code.clone();
                                        move |_| {
                                            *selected.write() = code.clone();
                                            *content.write() = load_ui_toml(&code);
                                            *dirty.write() = false;
                                            *save_msg.write() = None;
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                // File path hint
                div {
                    style: "font-size: 11px; color: var(--fs-color-text-muted); \
                            word-break: break-all;",
                    {
                        let path = ui_toml_path(&selected.read());
                        format!("{}", path.display())
                    }
                }
            }

            // ── Editor area ───────────────────────────────────────────────────
            div {
                style: "flex: 1; padding: 16px; display: flex; flex-direction: column; \
                        overflow: hidden;",

                // Header bar
                div {
                    style: "display: flex; justify-content: space-between; \
                            align-items: center; margin-bottom: 12px; flex-shrink: 0;",
                    h3 { style: "margin: 0; font-size: 15px;",
                        "{selected.read()}/ui.toml"
                        if *dirty.read() {
                            span { style: "margin-left: 6px; font-size: 12px; \
                                           color: var(--fs-color-warning, #f59e0b);",
                                {fs_i18n::t("builder.i18n.unsaved")}
                            }
                        }
                    }
                    div { style: "display: flex; gap: 8px; align-items: center;",
                        if let Some(msg) = save_msg.read().as_deref() {
                            span {
                                style: "font-size: 12px; color: var(--fs-color-text-muted);",
                                "{msg}"
                            }
                        }
                        button {
                            style: "padding: 6px 16px; background: var(--fs-color-primary); \
                                    color: white; border: none; \
                                    border-radius: var(--fs-radius-md); cursor: pointer; \
                                    font-size: 13px;",
                            disabled: !*dirty.read(),
                            onclick: move |_| {
                                let code = selected.read().clone();
                                let text = content.read().clone();
                                match save_ui_toml(&code, &text) {
                                    Ok(()) => {
                                        *dirty.write() = false;
                                        *save_msg.write() = Some("Saved.".to_string());
                                    }
                                    Err(e) => {
                                        *save_msg.write() = Some(format!("Error: {e}"));
                                    }
                                }
                            },
                            {fs_i18n::t("actions.save")}
                        }
                    }
                }

                // TOML textarea
                if content.read().is_empty() {
                    div {
                        style: "flex: 1; display: flex; align-items: center; \
                                justify-content: center; \
                                color: var(--fs-color-text-muted); font-size: 13px; \
                                text-align: center; padding: 32px; \
                                border: 1px dashed var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md);",
                        div {
                            p { style: "margin: 0 0 8px;", {fs_i18n::t("builder.i18n.no_toml_title")} }
                            p { style: "font-size: 12px;",
                                {fs_i18n::t("builder.i18n.no_toml_hint")}
                            }
                            code {
                                style: "font-size: 11px; display: block; margin-top: 8px; \
                                        background: var(--fs-color-bg-overlay); \
                                        padding: 6px 10px; border-radius: 4px;",
                                "~/.local/share/fsn/i18n/{selected.read()}/ui.toml"
                            }
                        }
                    }
                } else {
                    textarea {
                        style: "flex: 1; width: 100%; padding: 10px 12px; \
                                font-family: var(--fs-font-mono, monospace); font-size: 13px; \
                                border: 1px solid var(--fs-color-border-default); \
                                border-radius: var(--fs-radius-md); resize: none; \
                                background: var(--fs-color-bg-elevated); \
                                color: var(--fs-color-text-primary); \
                                box-sizing: border-box; min-height: 400px;",
                        value: "{content.read()}",
                        oninput: move |e| {
                            *content.write() = e.value();
                            *dirty.write() = true;
                            *save_msg.write() = None;
                        },
                    }
                }
            }
        }
    }
}

// ── LangBtn ────────────────────────────────────────────────────────────────────

#[component]
fn LangBtn(
    code:     String,
    name:     String,
    active:   bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let bg = if active {
        "background: var(--fs-color-primary); color: white;"
    } else {
        "background: transparent; color: var(--fs-color-text-primary);"
    };

    rsx! {
        button {
            style: "display: flex; justify-content: space-between; width: 100%; \
                    text-align: left; padding: 8px 10px; border: none; \
                    cursor: pointer; font-size: 13px; {bg}",
            onclick: on_click,
            span { "{name}" }
            span { style: "opacity: 0.6; font-size: 11px;", "{code}" }
        }
    }
}
