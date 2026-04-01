// view.rs — FsView implementation for ContainerAppModel.
//
// Only file in fs-container-app that imports fs-render.

use fs_render::{
    view::FsView,
    widget::{ButtonWidget, FsWidget, ListWidget},
};

use crate::model::ContainerAppModel;

// ── ContainerAppView ──────────────────────────────────────────────────────────

/// Snapshot view of the container app state.
pub struct ContainerAppView {
    pub model: ContainerAppModel,
}

impl ContainerAppView {
    #[must_use]
    pub fn new(model: ContainerAppModel) -> Self {
        Self { model }
    }
}

impl FsView for ContainerAppView {
    fn view(&self) -> Box<dyn FsWidget> {
        let refresh_btn = ButtonWidget {
            id: "container-btn-refresh".into(),
            label: "container-refresh".into(), // FTL key
            enabled: !self.model.loading,
            action: "refresh".into(),
        };

        let items: Vec<String> = std::iter::once(refresh_btn.label.clone())
            .chain(
                self.model
                    .containers
                    .iter()
                    .map(|c| format!("{} [{}]", c.name, c.state_label)),
            )
            .collect();

        Box::new(ListWidget {
            id: "container-list".into(),
            items,
            selected_index: self.model.active.as_ref().and_then(|a| {
                self.model
                        .containers
                        .iter()
                        .position(|c| &c.name == a)
                        // +1 for the refresh button at index 0
                        .map(|i| i + 1)
            }),
            enabled: !self.model.loading,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ContainerAppModel, ContainerEntry};

    #[test]
    fn empty_view_produces_widget() {
        let v = ContainerAppView::new(ContainerAppModel::new());
        let w = v.view();
        assert_eq!(w.widget_id(), "container-list");
        assert!(w.is_enabled());
    }

    #[test]
    fn loaded_view_shows_containers() {
        let mut m = ContainerAppModel::new();
        m.containers = vec![ContainerEntry {
            name: "kanidm".into(),
            state_label: "active".into(),
        }];
        let w = ContainerAppView::new(m).view();
        assert_eq!(w.widget_id(), "container-list");
    }

    #[test]
    fn loading_view_is_disabled() {
        let mut m = ContainerAppModel::new();
        m.loading = true;
        let w = ContainerAppView::new(m).view();
        assert!(!w.is_enabled());
    }
}
