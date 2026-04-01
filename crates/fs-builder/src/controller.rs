// controller.rs — BuilderController: MVC controller for the builder.
//
// Manages active pipelines. Each pipeline runs steps in sequence.

use std::sync::{Arc, Mutex};

use crate::model::{BuildPipeline, BuildStepStatus};

// ── BuilderController ─────────────────────────────────────────────────────────

/// Shared, cheaply-clonable controller for build pipelines.
#[derive(Clone, Default)]
pub struct BuilderController {
    pipelines: Arc<Mutex<Vec<BuildPipeline>>>,
}

impl BuilderController {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new build pipeline for the given package path.
    pub fn start(&self, package_path: impl Into<String>) -> BuildPipeline {
        let pipeline = BuildPipeline::new(package_path);
        self.pipelines.lock().unwrap().push(pipeline.clone());
        pipeline
    }

    /// Return snapshots of all pipelines.
    pub fn list(&self) -> Vec<BuildPipeline> {
        self.pipelines.lock().unwrap().clone()
    }

    /// Advance the current step of the pipeline at `index` to Done.
    pub fn advance_step(&self, index: usize) -> bool {
        let mut guard = self.pipelines.lock().unwrap();
        let Some(pipeline) = guard.get_mut(index) else {
            return false;
        };
        let step = pipeline.current_step;
        if let Some(s) = pipeline.steps.get_mut(step) {
            if s.status == BuildStepStatus::Pending || s.status == BuildStepStatus::Running {
                s.mark_done();
                pipeline.current_step += 1;
                return true;
            }
        }
        false
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_creates_pipeline() {
        let ctrl = BuilderController::new();
        let p = ctrl.start("/my/package");
        assert_eq!(p.package_path, "/my/package");
        assert_eq!(ctrl.list().len(), 1);
    }

    #[test]
    fn advance_step_progresses_pipeline() {
        let ctrl = BuilderController::new();
        ctrl.start("/pkg");
        assert!(ctrl.advance_step(0));
        let list = ctrl.list();
        assert_eq!(list[0].current_step, 1);
    }

    #[test]
    fn advance_step_out_of_range_returns_false() {
        let ctrl = BuilderController::new();
        assert!(!ctrl.advance_step(99));
    }
}
