// view.rs — FsView implementation for BuildPipeline.
//
// Only file in fs-builder that imports fs-render.
// Domain types (BuildPipeline, BuildStep) do NOT import fs-render.

use fs_render::{
    view::FsView,
    widget::{FsWidget, ListWidget},
};

use crate::model::{BuildPipeline, BuildStepStatus};

// ── BuilderView ───────────────────────────────────────────────────────────────

/// Renderer-agnostic view of the builder wizard state.
pub struct BuilderView {
    pub pipelines: Vec<BuildPipeline>,
}

impl BuilderView {
    #[must_use]
    pub fn new(pipelines: Vec<BuildPipeline>) -> Self {
        Self { pipelines }
    }
}

impl FsView for BuilderView {
    fn view(&self) -> Box<dyn FsWidget> {
        let items: Vec<String> = self
            .pipelines
            .iter()
            .flat_map(|p| {
                std::iter::once(format!("📦 {}", p.package_path)).chain(p.steps.iter().map(|s| {
                    let icon = match &s.status {
                        BuildStepStatus::Pending => "⬜",
                        BuildStepStatus::Running => "🔄",
                        BuildStepStatus::Done => "✅",
                        BuildStepStatus::Failed(_) => "❌",
                    };
                    format!("  {icon} {}", s.kind.label())
                }))
            })
            .collect();

        Box::new(ListWidget {
            id: "builder-wizard".into(),
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
    use crate::model::BuildPipeline;

    #[test]
    fn empty_view_produces_widget() {
        let v = BuilderView::new(vec![]);
        let w = v.view();
        assert_eq!(w.widget_id(), "builder-wizard");
        assert!(w.is_enabled());
    }

    #[test]
    fn pipeline_view_has_items() {
        let p = BuildPipeline::new("/my/pkg");
        let v = BuilderView::new(vec![p]);
        let w = v.view();
        assert_eq!(w.widget_id(), "builder-wizard");
    }
}
