/// Bot Manager — manage messenger accounts and bot configurations.
use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, FS_SIDEBAR_CSS};
use fs_i18n;

use crate::accounts_view::AccountsView;
use crate::context::BotManagerContext;
use crate::groups_view::GroupsView;
use crate::model::{BotKind, MessagingBot, MessagingBotsConfig};

/// Display properties for a `BotSection` variant — single source of truth.
struct SectionMeta {
    id:       &'static str,
    icon:     &'static str,
    i18n_key: &'static str,
}

/// Active section in the Bot Manager.
#[derive(Clone, PartialEq, Debug)]
pub enum BotSection {
    Accounts,
    Bots,
    Broadcast,
    Gatekeeper,
    Groups,
}

const ALL_SECTIONS: &[BotSection] = &[
    BotSection::Accounts,
    BotSection::Bots,
    BotSection::Broadcast,
    BotSection::Gatekeeper,
    BotSection::Groups,
];

impl BotSection {
    /// Single match block — all display properties in one place.
    fn meta(&self) -> SectionMeta {
        match self {
            Self::Accounts   => SectionMeta { id: "accounts",   icon: "🔑", i18n_key: "bots.section.accounts"   },
            Self::Bots       => SectionMeta { id: "bots",       icon: "🤖", i18n_key: "bots.section.bots"       },
            Self::Broadcast  => SectionMeta { id: "broadcast",  icon: "📢", i18n_key: "bots.section.broadcast"  },
            Self::Gatekeeper => SectionMeta { id: "gatekeeper", icon: "🔒", i18n_key: "bots.section.gatekeeper" },
            Self::Groups     => SectionMeta { id: "groups",     icon: "📁", i18n_key: "bots.section.groups"     },
        }
    }

    pub fn id(&self)    -> &str    { self.meta().id }
    pub fn icon(&self)  -> &str    { self.meta().icon }
    pub fn label(&self) -> String  { fs_i18n::t(self.meta().i18n_key).to_string() }

    /// No match needed — delegates to `id()` via ALL_SECTIONS.
    pub fn from_id(id: &str) -> Option<Self> {
        ALL_SECTIONS.iter().find(|s| s.id() == id).cloned()
    }
}

/// Root component of the Bot Manager.
#[component]
pub fn BotManagerApp() -> Element {
    let bots         = use_signal(MessagingBotsConfig::load);
    let selected_idx = use_signal(|| Some(0usize));
    let ctx          = BotManagerContext { bots, selected_idx };
    provide_context(ctx.clone());

    let mut active = use_signal(|| BotSection::Accounts);

    let sidebar_items: Vec<SidebarItem> = ALL_SECTIONS.iter()
        .map(|s| SidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    let bot_list      = ctx.bots.read().clone();
    let sel_idx       = *ctx.selected_idx.read();
    let selected      = sel_idx.and_then(|i| bot_list.get(i).cloned());
    let active_bot_id = sel_idx
        .and_then(|i| bot_list.get(i))
        .map(|b| b.id.clone())
        .unwrap_or_default();

    let bots_sidebar_items: Vec<SidebarItem> = bot_list.iter()
        .map(|b| SidebarItem::new(b.id.clone(), b.kind.icon().to_string(), b.name.clone()))
        .collect();

    let broadcast_bot  = ctx.bot_by_kind(&BotKind::Broadcast);
    let gatekeeper_bot = ctx.bot_by_kind(&BotKind::Gatekeeper);

    // Signal copies for closures (Signal<T>: Copy — shares the same reactive storage)
    let bots_sig         = ctx.bots;
    let mut sel_idx_sig  = ctx.selected_idx;

    rsx! {
        style { "{FS_SIDEBAR_CSS}" }
        div {
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fs-color-bg-base);",

            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fs-border); \
                        flex-shrink: 0; background: var(--fs-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fs-text-primary);",
                    {fs_i18n::t("bots.title")}
                }
            }

            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                Sidebar {
                    items:     sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = BotSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                div {
                    style: "flex: 1; overflow: auto; padding: 20px;",

                    match *active.read() {
                        BotSection::Accounts => rsx! {
                            AccountsView {}
                        },

                        BotSection::Bots => rsx! {
                            div { style: "display: flex; height: 100%; overflow: hidden;",
                                div {
                                    style: "width: 220px; border-right: 1px solid var(--fs-border); overflow-y: auto;",
                                    Sidebar {
                                        items: bots_sidebar_items,
                                        active_id: active_bot_id,
                                        on_select: move |id: String| {
                                            sel_idx_sig.set(bots_sig.read().iter().position(|b| b.id == id));
                                        },
                                    }
                                }
                                div { style: "flex: 1; overflow: auto; padding: 0 20px;",
                                    match selected {
                                        None => rsx! {
                                            div {
                                                style: "display: flex; align-items: center; \
                                                        justify-content: center; height: 200px; \
                                                        color: var(--fs-color-text-muted); font-size: 13px;",
                                                "Select a bot from the list"
                                            }
                                        },
                                        Some(bot) => rsx! {
                                            BotDetail {
                                                bot,
                                                on_update: {
                                                    let ctx = ctx.clone();
                                                    move |updated| ctx.update_selected(updated)
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        },

                        BotSection::Broadcast => rsx! {
                            match broadcast_bot {
                                Some((idx, bot)) => rsx! {
                                    crate::broadcast_view::BroadcastView {
                                        bot,
                                        on_update: {
                                            let ctx = ctx.clone();
                                            move |updated| ctx.update_bot(idx, updated)
                                        }
                                    }
                                },
                                None => rsx! {
                                    div {
                                        style: "color: var(--fs-color-text-muted); font-size: 13px;",
                                        "No Broadcast bot configured."
                                    }
                                },
                            }
                        },

                        BotSection::Gatekeeper => rsx! {
                            match gatekeeper_bot {
                                Some((idx, bot)) => rsx! {
                                    crate::gatekeeper_view::GatekeeperView {
                                        bot,
                                        on_update: {
                                            let ctx = ctx.clone();
                                            move |updated| ctx.update_bot(idx, updated)
                                        }
                                    }
                                },
                                None => rsx! {
                                    div {
                                        style: "color: var(--fs-color-text-muted); font-size: 13px;",
                                        "No Gatekeeper bot configured."
                                    }
                                },
                            }
                        },

                        BotSection::Groups => rsx! {
                            GroupsView {}
                        },
                    }
                }
            }
        }
    }
}

// ── BotDetail ─────────────────────────────────────────────────────────────────

/// Detail pane for a single bot — delegates rendering to `bot.kind.view()`.
/// No match block needed; adding a new bot type only requires a new `BotView` impl.
#[component]
fn BotDetail(bot: MessagingBot, on_update: EventHandler<MessagingBot>) -> Element {
    let status_color = if bot.enabled { "#22c55e" } else { "#64748b" };
    let status_label = if bot.enabled { "● Running" } else { "○ Stopped" };
    let view = bot.kind.view();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px;",

            div { style: "display: flex; align-items: center; gap: 12px;",
                span { style: "font-size: 28px;", "{bot.kind.icon()}" }
                div {
                    h2 { style: "margin: 0; font-size: 18px; color: var(--fs-color-text-primary);",
                        "{bot.name}"
                    }
                    span { style: "font-size: 12px; color: {status_color};", "{status_label}" }
                }
            }

            { view.render(bot, on_update) }
        }
    }
}
