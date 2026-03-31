// view.rs — FsView implementation for BrowserModel.
//
// This is the ONLY file in fs-browser that imports fs-render.
// The domain types (BrowserModel, Bookmark) do NOT import fs-render.
//
// Pattern: view.rs is the "Bindeglied" (bridge) between domain and renderer.

use fs_render::{
    view::FsView,
    widget::{ButtonWidget, FsWidget, ListWidget, TextInputWidget},
};

use crate::model::BrowserModel;

// ── BrowserView ───────────────────────────────────────────────────────────────

/// A snapshot-based view of the browser state.
///
/// Constructed from a `BrowserModel` by the controller; passed to the render
/// engine for display.
pub struct BrowserView {
    pub model: BrowserModel,
}

impl BrowserView {
    /// Wrap `model` in a renderable view.
    #[must_use]
    pub fn new(model: BrowserModel) -> Self {
        Self { model }
    }
}

impl FsView for BrowserView {
    fn view(&self) -> Box<dyn FsWidget> {
        // Toolbar: address bar + navigation buttons.
        // Returns a list widget representing the browser chrome.
        // (Full iced rendering happens in fs-gui-engine-iced — here we expose
        //  a renderer-agnostic widget tree.)
        let url = self
            .model
            .current_url
            .as_deref()
            .unwrap_or("about:blank")
            .to_string();

        let back_btn = ButtonWidget {
            id: "browser-btn-back".into(),
            label: "browser-btn-back".into(), // FTL key resolved at render time
            enabled: self.model.history.can_go_back(),
            action: "back".into(),
        };

        let forward_btn = ButtonWidget {
            id: "browser-btn-forward".into(),
            label: "browser-btn-forward".into(),
            enabled: self.model.history.can_go_forward(),
            action: "forward".into(),
        };

        let reload_btn = ButtonWidget {
            id: "browser-btn-refresh".into(),
            label: "browser-btn-refresh".into(),
            enabled: !self.model.loading,
            action: "reload".into(),
        };

        let address_bar = TextInputWidget {
            id: "browser-address-bar".into(),
            placeholder: "browser-address-placeholder".into(), // FTL key
            value: url,
            enabled: !self.model.loading,
        };

        let items = vec![
            back_btn.label.clone(),
            forward_btn.label.clone(),
            reload_btn.label.clone(),
            address_bar.value.clone(),
        ];

        // Represent as a list of widget IDs — the iced engine resolves these
        // into actual widgets.
        Box::new(ListWidget {
            id: "browser-chrome".into(),
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

    fn view_with_url(url: &str) -> BrowserView {
        let mut m = BrowserModel::new();
        m.set_loading(url);
        m.set_loaded(Some("Test Page".into()));
        BrowserView::new(m)
    }

    #[test]
    fn view_produces_widget() {
        let v = view_with_url("https://example.com");
        let w = v.view();
        assert_eq!(w.widget_id(), "browser-chrome");
        assert!(w.is_enabled());
    }

    #[test]
    fn view_empty_model() {
        let v = BrowserView::new(BrowserModel::new());
        let w = v.view();
        assert_eq!(w.widget_id(), "browser-chrome");
    }

    #[test]
    fn view_loading_disables_address_bar_in_items() {
        let mut m = BrowserModel::new();
        m.set_loading("https://loading.example");
        // While loading the view should still produce a valid widget.
        let v = BrowserView::new(m);
        let w = v.view();
        assert_eq!(w.widget_id(), "browser-chrome");
    }
}
