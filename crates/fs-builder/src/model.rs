// model.rs — BuildPipeline: Chain of Responsibility.
//
// A pipeline is a sequence of BuildSteps:
//   Analyse → Validate → Build → Publish

use serde::{Deserialize, Serialize};

// ── BuildStepKind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildStepKind {
    Analyse,
    Validate,
    Build,
    Publish,
}

impl BuildStepKind {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Analyse => "builder-step-analyse",   // FTL key
            Self::Validate => "builder-step-validate", // FTL key
            Self::Build => "builder-step-build",       // FTL key
            Self::Publish => "builder-step-publish",   // FTL key
        }
    }
}

// ── BuildStepStatus ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildStepStatus {
    Pending,
    Running,
    Done,
    Failed(String),
}

impl BuildStepStatus {
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed(_))
    }
}

// ── BuildStep ─────────────────────────────────────────────────────────────────

/// A single step in the build pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BuildStep {
    pub kind: BuildStepKind,
    pub status: BuildStepStatus,
    pub message: Option<String>,
}

impl BuildStep {
    #[must_use]
    pub fn new(kind: BuildStepKind) -> Self {
        Self {
            kind,
            status: BuildStepStatus::Pending,
            message: None,
        }
    }

    pub fn mark_running(&mut self) {
        self.status = BuildStepStatus::Running;
    }

    pub fn mark_done(&mut self) {
        self.status = BuildStepStatus::Done;
    }

    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.status = BuildStepStatus::Failed(reason.into());
    }
}

// ── BuildPipeline ─────────────────────────────────────────────────────────────

/// A build pipeline — ordered Chain of Responsibility.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BuildPipeline {
    pub package_path: String,
    pub steps: Vec<BuildStep>,
    pub current_step: usize,
}

impl BuildPipeline {
    #[must_use]
    pub fn new(package_path: impl Into<String>) -> Self {
        Self {
            package_path: package_path.into(),
            steps: vec![
                BuildStep::new(BuildStepKind::Analyse),
                BuildStep::new(BuildStepKind::Validate),
                BuildStep::new(BuildStepKind::Build),
                BuildStep::new(BuildStepKind::Publish),
            ],
            current_step: 0,
        }
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| s.status.is_terminal())
    }

    #[must_use]
    pub fn has_failed(&self) -> bool {
        self.steps
            .iter()
            .any(|s| matches!(s.status, BuildStepStatus::Failed(_)))
    }

    #[must_use]
    pub fn current(&self) -> Option<&BuildStep> {
        self.steps.get(self.current_step)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pipeline_has_four_steps() {
        let p = BuildPipeline::new("/pkg/hello");
        assert_eq!(p.steps.len(), 4);
    }

    #[test]
    fn new_pipeline_is_not_complete() {
        let p = BuildPipeline::new("/pkg");
        assert!(!p.is_complete());
    }

    #[test]
    fn all_done_means_complete() {
        let mut p = BuildPipeline::new("/pkg");
        for s in &mut p.steps {
            s.mark_done();
        }
        assert!(p.is_complete());
    }

    #[test]
    fn failed_step_detected() {
        let mut p = BuildPipeline::new("/pkg");
        p.steps[0].mark_failed("missing manifest");
        assert!(p.has_failed());
    }
}
