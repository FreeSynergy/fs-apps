// view.rs — FsView implementation for BotsView.
//
// Only file in fs-bots that imports fs-render.
// Domain types (MessagingBot, MessagingBotsConfig) do NOT import fs-render.

use fs_render::{
    view::FsView,
    widget::{FsWidget, ListWidget},
};

use crate::model::MessagingBotsConfig;

// ── BotsView ──────────────────────────────────────────────────────────────────

/// Renderer-agnostic view of the bot list.
pub struct BotsView {
    pub config: MessagingBotsConfig,
}

impl BotsView {
    #[must_use]
    pub fn new(config: MessagingBotsConfig) -> Self {
        Self { config }
    }
}

impl FsView for BotsView {
    fn view(&self) -> Box<dyn FsWidget> {
        let items: Vec<String> = self
            .config
            .bots
            .iter()
            .map(|b| {
                let status = if b.enabled { "✓" } else { "✗" };
                format!("{status} [{}] {}", b.kind.meta().icon, b.name)
            })
            .collect();

        Box::new(ListWidget {
            id: "bots-list".into(),
            items,
            selected_index: None,
            enabled: true,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_view_produces_widget() {
        let v = BotsView::new(MessagingBotsConfig::default());
        let w = v.view();
        assert_eq!(w.widget_id(), "bots-list");
        assert!(w.is_enabled());
    }
}
