use dioxus::prelude::*;
use chrono::Utc;
use crate::components::EmptyState;
use crate::model::{ApprovalAction, MessagingBot, PendingApproval};

#[component]
pub fn GatekeeperView(bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
    let approvals = bot.pending_approvals.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div {
                div {
                    style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                            letter-spacing: 0.06em; color: var(--fs-color-text-muted); margin-bottom: 8px;",
                    "Pending Verification"
                }

                if approvals.is_empty() {
                    EmptyState { message: "No pending verifications".to_string() }
                }

                div { style: "display: flex; flex-direction: column; gap: 6px;",
                    for approval in approvals.iter() {
                        ApprovalRow {
                            key: "{approval.id}",
                            approval: approval.clone(),
                            bot: bot.clone(),
                            on_update,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ApprovalRow(
    approval:  PendingApproval,
    bot:       MessagingBot,
    on_update: EventHandler<MessagingBot>,
) -> Element {
    let secs    = (Utc::now() - approval.waiting_since).num_seconds();
    let waiting = if secs < 60 { format!("{secs}s") } else { format!("{}m", secs / 60) };
    let id      = approval.id.clone();

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; \
                    background: var(--fs-color-bg-overlay); \
                    border-radius: var(--fs-radius-md); \
                    padding: 10px 14px; font-size: 13px;",
            span { style: "font-size: 16px;", "🔒" }
            span { style: "flex: 1; color: var(--fs-color-text-primary); font-weight: 500;",
                "{approval.username}"
            }
            span { style: "color: var(--fs-color-text-muted); font-size: 11px;",
                "{approval.platform} — waiting {waiting}"
            }
            button {
                onclick: {
                    let mut b = bot.clone();
                    let id    = id.clone();
                    move |_| {
                        b.resolve_approval(&id, ApprovalAction::Allow);
                        on_update.call(b.clone());
                    }
                },
                style: "background: #22c55e; color: #fff; \
                        border: none; border-radius: var(--fs-radius-sm); \
                        padding: 5px 12px; font-size: 12px; cursor: pointer;",
                "✓ Allow"
            }
            button {
                onclick: {
                    let mut b = bot.clone();
                    move |_| {
                        b.resolve_approval(&id, ApprovalAction::Deny);
                        on_update.call(b.clone());
                    }
                },
                style: "background: #ef4444; color: #fff; \
                        border: none; border-radius: var(--fs-radius-sm); \
                        padding: 5px 12px; font-size: 12px; cursor: pointer;",
                "✗ Deny"
            }
        }
    }
}
