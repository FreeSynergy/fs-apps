/// BotView trait — decouples bot-kind dispatch from the BotDetail component.
/// Each BotKind returns its own `Box<dyn BotView>`; `BotDetail` calls `view.render()`
/// instead of a match block, so new bot types only require a new impl here.
use dioxus::prelude::*;

use crate::broadcast_view::BroadcastView;
use crate::gatekeeper_view::GatekeeperView;
use crate::model::{BotKind, MessagingBot};

// ── BotView ────────────────────────────────────────────────────────────────────

/// Trait for bot-type-specific view rendering.
pub trait BotView {
    fn render(&self, bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element;
    fn icon(&self)  -> &'static str;
    fn label(&self) -> &'static str;
}

// ── BroadcastBotView ──────────────────────────────────────────────────────────

pub struct BroadcastBotView;

impl BotView for BroadcastBotView {
    fn render(&self, bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
        rsx! { BroadcastView { bot, on_update } }
    }
    fn icon(&self)  -> &'static str { "📢" }
    fn label(&self) -> &'static str { "Broadcast" }
}

// ── GatekeeperBotView ─────────────────────────────────────────────────────────

pub struct GatekeeperBotView;

impl BotView for GatekeeperBotView {
    fn render(&self, bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
        rsx! { GatekeeperView { bot, on_update } }
    }
    fn icon(&self)  -> &'static str { "🔒" }
    fn label(&self) -> &'static str { "Gatekeeper" }
}

// ── DefaultBotView ────────────────────────────────────────────────────────────

pub struct DefaultBotView;

impl BotView for DefaultBotView {
    fn render(&self, _bot: MessagingBot, _on_update: EventHandler<MessagingBot>) -> Element {
        rsx! {
            div {
                style: "background: var(--fs-color-bg-overlay); \
                        border-radius: var(--fs-radius-md); \
                        padding: 20px; color: var(--fs-color-text-muted); font-size: 13px;",
                "This bot type does not have a usage interface yet."
            }
        }
    }
    fn icon(&self)  -> &'static str { "🤖" }
    fn label(&self) -> &'static str { "Bot" }
}

// ── BotKind extension ──────────────────────────────────────────────────────────

impl BotKind {
    /// Return the view implementation for this bot kind.
    /// Replaces the `match kind { ... }` block previously in `BotDetail`.
    pub fn view(&self) -> Box<dyn BotView> {
        match self {
            Self::Broadcast  => Box::new(BroadcastBotView),
            Self::Gatekeeper => Box::new(GatekeeperBotView),
            _                => Box::new(DefaultBotView),
        }
    }
}
