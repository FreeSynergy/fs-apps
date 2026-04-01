// view.rs — FsView implementation for AiModel.
//
// Only file in fs-ai that imports fs-render.

use fs_render::{
    view::FsView,
    widget::{ButtonWidget, FsWidget, ListWidget},
};

use crate::model::{AiModel, KnownModel};

// ── AiView ────────────────────────────────────────────────────────────────────

/// Snapshot view of the AI assistant state.
pub struct AiView {
    pub model: AiModel,
    pub models: Vec<KnownModel>,
}

impl AiView {
    #[must_use]
    pub fn new(model: AiModel, models: Vec<KnownModel>) -> Self {
        Self { model, models }
    }
}

impl FsView for AiView {
    fn view(&self) -> Box<dyn FsWidget> {
        let start_btn = ButtonWidget {
            id: "ai-btn-start".into(),
            label: "ai-start-engine".into(), // FTL key
            enabled: !self.model.running && !self.model.busy,
            action: "start".into(),
        };

        let stop_btn = ButtonWidget {
            id: "ai-btn-stop".into(),
            label: "ai-stop-engine".into(), // FTL key
            enabled: self.model.running && !self.model.busy,
            action: "stop".into(),
        };

        let model_names: Vec<String> = self.models.iter().map(|m| m.name.clone()).collect();

        let status_label = if self.model.running {
            format!("ai-status-running:port={}", self.model.port.unwrap_or(0))
        } else {
            "ai-status-stopped".into()
        };

        Box::new(ListWidget {
            id: "ai-panel".into(),
            items: std::iter::once(status_label)
                .chain(std::iter::once(start_btn.label.clone()))
                .chain(std::iter::once(stop_btn.label.clone()))
                .chain(model_names)
                .collect(),
            selected_index: None,
            enabled: !self.model.busy,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AiModel, KnownModel};

    fn make_view(running: bool) -> AiView {
        let mut model = AiModel::new();
        if running {
            model.set_running(1234);
        }
        AiView::new(model, KnownModel::all())
    }

    #[test]
    fn view_produces_widget() {
        let v = make_view(false);
        let w = v.view();
        assert_eq!(w.widget_id(), "ai-panel");
        assert!(w.is_enabled());
    }

    #[test]
    fn running_view_produces_widget() {
        let v = make_view(true);
        let w = v.view();
        assert_eq!(w.widget_id(), "ai-panel");
    }

    #[test]
    fn busy_view_is_disabled() {
        let mut model = AiModel::new();
        model.busy = true;
        let v = AiView::new(model, KnownModel::all());
        let w = v.view();
        assert!(!w.is_enabled());
    }
}
