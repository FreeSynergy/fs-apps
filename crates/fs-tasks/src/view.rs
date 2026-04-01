// view.rs — FsView impl for fs-tasks. Only file allowed to import fs-render.

use fs_render::{
    view::FsView,
    widget::{ButtonWidget, FsWidget, ListWidget},
};

use crate::model::TaskPipeline;

// ── TasksView ─────────────────────────────────────────────────────────────────

/// View for the task pipeline list.
pub struct TasksView {
    pub tasks: Vec<TaskPipeline>,
}

impl FsView for TasksView {
    fn view(&self) -> Box<dyn FsWidget> {
        let items: Vec<String> = self
            .tasks
            .iter()
            .map(|t| {
                format!(
                    "{} [{}] {} → {}",
                    t.name,
                    if t.enabled { "on" } else { "off" },
                    t.source.service,
                    t.target.service,
                )
            })
            .collect();
        Box::new(ListWidget {
            id: "tasks-list".into(),
            items,
            selected_index: None,
            enabled: true,
        })
    }
}

// ── TaskDetailView ────────────────────────────────────────────────────────────

/// View for a single task pipeline's detail panel.
pub struct TaskDetailView {
    pub task: TaskPipeline,
}

impl FsView for TaskDetailView {
    fn view(&self) -> Box<dyn FsWidget> {
        let label = format!(
            "{} — {} → {}  [{}]  {}",
            self.task.name,
            self.task.source.service,
            self.task.target.service,
            self.task.trigger.label(),
            self.task.status_label(),
        );
        Box::new(ListWidget {
            id: "task-detail".into(),
            items: vec![label],
            selected_index: None,
            enabled: false,
        })
    }
}

// ── CreateTaskView ────────────────────────────────────────────────────────────

/// View with a "Create task" action button.
pub struct CreateTaskView;

impl FsView for CreateTaskView {
    fn view(&self) -> Box<dyn FsWidget> {
        Box::new(ButtonWidget {
            id: "create-task".into(),
            label: "tasks-create".into(), // FTL key
            enabled: true,
            action: "create".into(),
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task() -> TaskPipeline {
        TaskPipeline::new_default(1)
    }

    #[test]
    fn tasks_view_empty() {
        let v = TasksView { tasks: vec![] };
        let w = v.view();
        assert_eq!(w.widget_id(), "tasks-list");
        assert!(w.is_enabled());
    }

    #[test]
    fn tasks_view_with_task() {
        let v = TasksView {
            tasks: vec![sample_task()],
        };
        let w = v.view();
        assert_eq!(w.widget_id(), "tasks-list");
    }

    #[test]
    fn task_detail_view() {
        let v = TaskDetailView {
            task: sample_task(),
        };
        let w = v.view();
        assert_eq!(w.widget_id(), "task-detail");
    }

    #[test]
    fn create_task_view() {
        let v = CreateTaskView;
        let w = v.view();
        assert_eq!(w.widget_id(), "create-task");
        assert!(w.is_enabled());
    }
}
