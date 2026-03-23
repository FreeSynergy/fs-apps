/// Accounts view — manage Control Bot messenger account connections.
use dioxus::prelude::*;

use crate::components::{EmptyState, StatusDot};
use crate::model::{ControlBotAccount, ControlBotConfig, Platform, TomlConfig};

#[component]
pub fn AccountsView() -> Element {
    let mut accounts: Signal<Vec<ControlBotAccount>> = use_signal(ControlBotConfig::load);
    let mut show_form     = use_signal(|| false);
    let mut form_platform = use_signal(|| Platform::Telegram);
    let mut form_label    = use_signal(String::new);
    let mut form_creds: Signal<Vec<String>> = use_signal(Vec::new);

    let platform_fields = form_platform.read().credential_fields();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px; max-width: 700px;",

            // Control Bot concept banner
            div {
                style: "padding: 16px; border-radius: var(--fs-radius-md, 8px); \
                        background: var(--fs-color-bg-surface, #1e293b); \
                        border: 1px solid var(--fs-color-border-default, #334155);",
                h3 {
                    style: "margin: 0 0 8px; font-size: 14px; font-weight: 700; \
                            color: var(--fs-color-text-primary);",
                    "Control Bot — Master Messenger Connection"
                }
                p {
                    style: "margin: 0 0 4px; font-size: 13px; color: var(--fs-color-text-muted);",
                    "Connect your messenger accounts once. The Control Bot connects to all of them,"
                }
                p {
                    style: "margin: 0; font-size: 13px; color: var(--fs-color-text-muted);",
                    "and all bots (Broadcast, Gatekeeper, etc.) work through it automatically."
                }
            }

            // Account list
            div { style: "display: flex; flex-direction: column; gap: 8px;",
                if accounts.read().is_empty() {
                    EmptyState { message: "No accounts configured yet.".to_string() }
                } else {
                    for (idx, account) in accounts.read().clone().iter().enumerate() {
                        AccountRow {
                            key: "{account.id}",
                            account: account.clone(),
                            on_remove: move |_| {
                                accounts.write().remove(idx);
                                let _ = ControlBotConfig { accounts: accounts.read().clone() }.save();
                            },
                        }
                    }
                }
            }

            // Add Account button
            if !*show_form.read() {
                button {
                    style: "align-self: flex-start; \
                            background: var(--fs-color-primary, #00bcd4); color: #fff; \
                            border: none; border-radius: var(--fs-radius-md, 6px); \
                            padding: 9px 18px; font-size: 13px; font-weight: 600; cursor: pointer;",
                    onclick: move |_| {
                        form_platform.set(Platform::Telegram);
                        form_label.set(String::new());
                        form_creds.set(vec!["".to_string(); Platform::Telegram.credential_fields().len()]);
                        show_form.set(true);
                    },
                    "+ Add Account"
                }
            }

            // Inline add form
            if *show_form.read() {
                div {
                    style: "padding: 20px; border-radius: var(--fs-radius-md, 8px); \
                            background: var(--fs-color-bg-surface, #1e293b); \
                            border: 1px solid var(--fs-color-primary, #00bcd4);",

                    div { style: "display: flex; flex-direction: column; gap: 14px;",

                        // Platform selector
                        div {
                            label {
                                style: "display: block; font-size: 12px; font-weight: 600; \
                                        color: var(--fs-color-text-muted); margin-bottom: 6px; \
                                        text-transform: uppercase; letter-spacing: 0.06em;",
                                "Platform"
                            }
                            select {
                                style: "width: 100%; background: var(--fs-color-bg-base, #0f172a); \
                                        border: 1px solid var(--fs-color-border-default, #334155); \
                                        border-radius: var(--fs-radius-md, 6px); \
                                        padding: 8px 10px; font-size: 13px; \
                                        color: var(--fs-color-text-primary);",
                                onchange: move |e| {
                                    let p = Platform::from_str(&e.value());
                                    let field_count = p.credential_fields().len();
                                    form_platform.set(p);
                                    form_creds.set(vec!["".to_string(); field_count]);
                                },
                                for p in Platform::all() {
                                    option {
                                        value: "{p.label()}",
                                        selected: *form_platform.read() == *p,
                                        "{p.icon()} {p.label()}"
                                    }
                                }
                            }
                        }

                        // Label
                        div {
                            label {
                                style: "display: block; font-size: 12px; font-weight: 600; \
                                        color: var(--fs-color-text-muted); margin-bottom: 6px; \
                                        text-transform: uppercase; letter-spacing: 0.06em;",
                                "Label"
                            }
                            input {
                                r#type: "text",
                                placeholder: "e.g. My Telegram",
                                style: "width: 100%; background: var(--fs-color-bg-base, #0f172a); \
                                        border: 1px solid var(--fs-color-border-default, #334155); \
                                        border-radius: var(--fs-radius-md, 6px); \
                                        padding: 8px 10px; font-size: 13px; \
                                        color: var(--fs-color-text-primary); box-sizing: border-box;",
                                value: "{form_label.read()}",
                                oninput: move |e| form_label.set(e.value()),
                            }
                        }

                        // Dynamic credential fields
                        for (field_idx, field) in platform_fields.iter().enumerate() {
                            div {
                                key: "{field.name}",
                                label {
                                    style: "display: block; font-size: 12px; font-weight: 600; \
                                            color: var(--fs-color-text-muted); margin-bottom: 6px; \
                                            text-transform: uppercase; letter-spacing: 0.06em;",
                                    "{field.name}"
                                }
                                input {
                                    r#type: if field.is_secret { "password" } else { "text" },
                                    placeholder: "{field.placeholder}",
                                    style: "width: 100%; background: var(--fs-color-bg-base, #0f172a); \
                                            border: 1px solid var(--fs-color-border-default, #334155); \
                                            border-radius: var(--fs-radius-md, 6px); \
                                            padding: 8px 10px; font-size: 13px; \
                                            color: var(--fs-color-text-primary); box-sizing: border-box;",
                                    value: {
                                        let v = form_creds.read();
                                        v.get(field_idx).cloned().unwrap_or_default()
                                    },
                                    oninput: move |e| {
                                        let mut creds = form_creds.write();
                                        while creds.len() <= field_idx { creds.push(String::new()); }
                                        creds[field_idx] = e.value();
                                    },
                                }
                            }
                        }

                        // Save / Cancel
                        div { style: "display: flex; gap: 10px;",
                            button {
                                style: "background: var(--fs-color-primary, #00bcd4); color: #fff; \
                                        border: none; border-radius: var(--fs-radius-md, 6px); \
                                        padding: 8px 18px; font-size: 13px; font-weight: 600; cursor: pointer;",
                                onclick: move |_| {
                                    let platform = form_platform.read().clone();
                                    let label    = form_label.read().clone();
                                    let fields   = platform.credential_fields();
                                    let creds_read = form_creds.read();
                                    let credentials: Vec<(String, String)> = fields.iter().enumerate()
                                        .map(|(i, f)| (
                                            f.name.to_string(),
                                            creds_read.get(i).cloned().unwrap_or_default(),
                                        ))
                                        .collect();

                                    let count = accounts.read().len();
                                    if let Some(account) = ControlBotAccount::create(platform, label, credentials, count) {
                                        accounts.write().push(account);
                                        let _ = ControlBotConfig { accounts: accounts.read().clone() }.save();
                                        show_form.set(false);
                                    }
                                },
                                "Save"
                            }
                            button {
                                style: "background: transparent; \
                                        border: 1px solid var(--fs-color-border-default, #334155); \
                                        border-radius: var(--fs-radius-md, 6px); \
                                        padding: 8px 18px; font-size: 13px; \
                                        color: var(--fs-color-text-muted); cursor: pointer;",
                                onclick: move |_| show_form.set(false),
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AccountRow(account: ControlBotAccount, on_remove: EventHandler<()>) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; \
                    padding: 12px 14px; border-radius: var(--fs-radius-md, 8px); \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    border: 1px solid var(--fs-color-border-default, #334155);",

            span { style: "font-size: 20px; flex-shrink: 0;", "{account.platform.icon()}" }

            div { style: "flex: 1;",
                div {
                    style: "font-size: 13px; font-weight: 600; color: var(--fs-color-text-primary);",
                    "{account.label}"
                }
                div {
                    style: "font-size: 12px; color: var(--fs-color-text-muted);",
                    "{account.platform.label()}"
                }
            }

            StatusDot { connected: account.connected }

            button {
                style: "padding: 4px 12px; background: transparent; \
                        border: 1px solid var(--fs-color-border-default, #334155); \
                        border-radius: var(--fs-radius-md, 6px); \
                        font-size: 12px; color: var(--fs-color-text-muted); cursor: pointer;",
                "Edit"
            }

            button {
                style: "padding: 4px 8px; background: transparent; \
                        border: 1px solid rgba(239,68,68,0.4); \
                        border-radius: var(--fs-radius-md, 6px); \
                        font-size: 12px; color: #ef4444; cursor: pointer;",
                onclick: move |_| on_remove.call(()),
                "✕"
            }
        }
    }
}
