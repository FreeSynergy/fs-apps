// controller.rs — shared state / domain logic for fs-lenses.

use crate::model::Lens;

/// Shared, cheaply-clonable controller for lens operations.
#[derive(Clone, Default)]
pub struct LensController {
    lenses: std::sync::Arc<std::sync::Mutex<Vec<Lens>>>,
}

impl LensController {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<Lens> {
        self.lenses.lock().unwrap().clone()
    }

    pub fn create(&self, name: String, query: String) -> Lens {
        let lens = Lens::new(name, query);
        self.lenses.lock().unwrap().push(lens.clone());
        lens
    }

    pub fn delete(&self, id: i64) -> bool {
        let mut guard = self.lenses.lock().unwrap();
        let before = guard.len();
        guard.retain(|l| l.id != id);
        guard.len() < before
    }

    pub fn query(&self, _lens_id: i64, _query: &str) -> Vec<String> {
        // Stub: real implementation would dispatch to the bus.
        vec![]
    }
}
