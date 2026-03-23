use dioxus::prelude::*;
use crate::components::{ListRow, ListRowContent};
use crate::model::{BroadcastRecord, ChannelTarget, MessagingBot};

// ── ListRowContent impls ───────────────────────────────────────────────────────

impl ListRowContent for ChannelTarget {
    fn row_icon(&self) -> Element {
        let check = if self.enabled { "☑" } else { "☐" };
        rsx! { span { style: "color: var(--fs-color-primary);", "{check}" } }
    }
    fn row_body(&self) -> Element {
        rsx! {
            span { style: "color: var(--fs-color-text-muted); font-size: 11px;", "{self.platform}:" }
            span { style: "color: var(--fs-color-text-primary);", "{self.name}" }
        }
    }
    fn row_trailing(&self) -> Element { rsx! { } }
}

impl ListRowContent for BroadcastRecord {
    fn row_icon(&self) -> Element {
        rsx! { span { "📤" } }
    }
    fn row_body(&self) -> Element {
        let preview  = self.preview(50);
        let time_ago = self.time_ago();
        rsx! {
            span {
                style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; \
                        color: var(--fs-color-text-primary);",
                "\"{preview}\""
            }
            span { "— {time_ago}" }
        }
    }
    fn row_trailing(&self) -> Element {
        rsx! { span { "— {self.target_count} targets" } }
    }
}

// ── BroadcastView ─────────────────────────────────────────────────────────────

#[component]
pub fn BroadcastView(bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
    let mut message = use_signal(String::new);
    let targets      = bot.targets.clone();
    let recent       = bot.recent_broadcasts.clone();
    let enabled_count = targets.iter().filter(|t| t.enabled).count();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",

            div {
                label {
                    style: "display: block; font-size: 12px; font-weight: 600; \
                            color: var(--fs-color-text-muted); margin-bottom: 6px; \
                            text-transform: uppercase; letter-spacing: 0.06em;",
                    "Message"
                }
                textarea {
                    style: "width: 100%; min-height: 100px; \
                            background: var(--fs-color-bg-overlay); \
                            border: 1px solid var(--fs-color-border-default); \
                            border-radius: var(--fs-radius-md); \
                            padding: 10px 12px; font-size: 13px; font-family: inherit; \
                            color: var(--fs-color-text-primary); resize: vertical; \
                            box-sizing: border-box;",
                    placeholder: "Type your broadcast message…",
                    oninput: move |e| message.set(e.value()),
                }
            }

            div {
                label {
                    style: "display: block; font-size: 12px; font-weight: 600; \
                            color: var(--fs-color-text-muted); margin-bottom: 8px; \
                            text-transform: uppercase; letter-spacing: 0.06em;",
                    "Send to"
                }
                div { style: "display: flex; flex-direction: column;",
                    for t in &targets {
                        ListRow { key: "{t.platform}:{t.id}", item: t.clone() }
                    }
                }
            }

            button {
                style: "align-self: flex-start; background: var(--fs-color-primary); \
                        color: #fff; border: none; border-radius: var(--fs-radius-md); \
                        padding: 10px 20px; font-size: 14px; font-weight: 600; \
                        cursor: pointer;",
                onclick: {
                    let msg = message.read().clone();
                    let mut updated = bot.clone();
                    move |_| {
                        if updated.send_broadcast(&msg, enabled_count) {
                            on_update.call(updated.clone());
                        }
                    }
                },
                "📤 Send Broadcast"
            }

            if !recent.is_empty() {
                div {
                    div {
                        style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                                letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 8px;",
                        "Recent Broadcasts"
                    }
                    div { style: "display: flex; flex-direction: column;",
                        for record in &recent {
                            ListRow { key: "{record.sent_at}", item: record.clone() }
                        }
                    }
                }
            }
        }
    }
}
