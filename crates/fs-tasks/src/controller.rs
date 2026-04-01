// controller.rs — shared state / domain logic for fs-tasks.

use std::sync::{Arc, Mutex};

use crate::model::{TaskPipeline, TasksConfig};

/// Shared, cheaply-clonable controller for task pipeline operations.
#[derive(Clone)]
pub struct TaskController {
    tasks: Arc<Mutex<Vec<TaskPipeline>>>,
}

impl TaskController {
    #[must_use]
    pub fn new() -> Self {
        let tasks = TasksConfig::load().tasks;
        Self {
            tasks: Arc::new(Mutex::new(tasks)),
        }
    }

    pub fn list(&self) -> Vec<TaskPipeline> {
        self.tasks.lock().unwrap().clone()
    }

    pub fn create(&self, name: String) -> TaskPipeline {
        let mut guard = self.tasks.lock().unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let count = guard.len() as u32;
        let mut task = TaskPipeline::new_default(count + 1);
        task.name = name;
        guard.push(task.clone());
        task
    }

    pub fn delete(&self, id: &str) -> bool {
        let mut guard = self.tasks.lock().unwrap();
        let before = guard.len();
        guard.retain(|t| t.id != id);
        guard.len() < before
    }

    /// Flips the `enabled` field. Returns the new enabled value, or `None` if not found.
    pub fn toggle(&self, id: &str) -> Option<bool> {
        let mut guard = self.tasks.lock().unwrap();
        guard.iter_mut().find(|t| t.id == id).map(|t| {
            t.enabled = !t.enabled;
            t.enabled
        })
    }
}

impl Default for TaskController {
    fn default() -> Self {
        Self::new()
    }
}
