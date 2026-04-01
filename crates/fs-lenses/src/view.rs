// view.rs — FsView implementation for LensController.
//
// This is the ONLY file in fs-lenses that imports fs-render.
// Domain types (Lens, LensController) do NOT import fs-render.

use fs_render::{
    view::FsView,
    widget::{ButtonWidget, FsWidget, ListWidget, TextInputWidget},
};

use crate::model::Lens;

// ── LensesView ────────────────────────────────────────────────────────────────

/// Snapshot view of the lenses state.
///
/// Constructed from a slice of lenses; passed to the render engine for display.
pub struct LensesView {
    pub lenses: Vec<Lens>,
}

impl LensesView {
    /// Wrap a lens snapshot in a renderable view.
    #[must_use]
    pub fn new(lenses: Vec<Lens>) -> Self {
        Self { lenses }
    }
}

impl FsView for LensesView {
    fn view(&self) -> Box<dyn FsWidget> {
        // Each lens becomes one item in the list.
        let items: Vec<String> = self
            .lenses
            .iter()
            .map(|l| format!("{}: {}", l.name, l.query))
            .collect();

        Box::new(ListWidget {
            id: "lenses-list".into(),
            items,
            selected_index: None,
            enabled: true,
        })
    }
}

// ── LensDetailView ────────────────────────────────────────────────────────────

/// View for a single lens and its query results.
pub struct LensDetailView {
    pub lens: Lens,
}

impl LensDetailView {
    #[must_use]
    pub fn new(lens: Lens) -> Self {
        Self { lens }
    }
}

impl FsView for LensDetailView {
    fn view(&self) -> Box<dyn FsWidget> {
        let search_btn = ButtonWidget {
            id: "lens-search-btn".into(),
            label: "lenses-search-btn".into(), // FTL key resolved at render time
            enabled: !self.lens.loading,
            action: "search".into(),
        };

        let query_input = TextInputWidget {
            id: "lens-query-input".into(),
            placeholder: "lenses-search-placeholder".into(), // FTL key
            value: self.lens.query.clone(),
            enabled: !self.lens.loading,
        };

        let result_items: Vec<String> = self
            .lens
            .items
            .iter()
            .map(|item| format!("[{}] {}", item.role.id(), item.summary))
            .collect();

        Box::new(ListWidget {
            id: format!("lens-detail-{}", self.lens.id),
            items: vec![query_input.value.clone(), search_btn.label.clone()]
                .into_iter()
                .chain(result_items)
                .collect(),
            selected_index: None,
            enabled: true,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Lens;

    #[test]
    fn empty_view_produces_widget() {
        let v = LensesView::new(vec![]);
        let w = v.view();
        assert_eq!(w.widget_id(), "lenses-list");
        assert!(w.is_enabled());
    }

    #[test]
    fn view_shows_all_lenses() {
        let lenses = vec![
            Lens::new("Alpha", "alpha query"),
            Lens::new("Beta", "beta query"),
        ];
        let v = LensesView::new(lenses);
        let w = v.view();
        assert_eq!(w.widget_id(), "lenses-list");
    }

    #[test]
    fn detail_view_has_correct_id() {
        let lens = Lens::new("Test", "test query");
        let id = lens.id;
        let v = LensDetailView::new(lens);
        let w = v.view();
        assert_eq!(w.widget_id(), format!("lens-detail-{id}"));
    }

    #[test]
    fn detail_view_loading_produces_widget() {
        let mut lens = Lens::new("Loading", "query");
        lens.loading = true;
        let v = LensDetailView::new(lens);
        let w = v.view();
        assert!(w.widget_id().starts_with("lens-detail-"));
    }
}
