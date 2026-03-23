// Lens query engine — queries the bus for data matching a lens's search term.
//
// Phase L2 implementation: publishes a `lens.query` event to the bus and
// collects responses from services. For the current phase, we produce
// mock/stub results until real bus routing is wired up.

use crate::model::{Lens, LensItem, LensRole};

// ── LensQueryEngine ───────────────────────────────────────────────────────────

pub struct LensQueryEngine;

impl LensQueryEngine {
    /// Query all services for data matching `lens.query` via the bus.
    ///
    /// Returns updated items. If the bus is unreachable, returns empty.
    pub async fn refresh_lens(&self, lens: &Lens) -> Vec<LensItem> {
        if let Ok(items) = self.query_via_bus(&lens.query).await {
            return items;
        }
        self.demo_items(&lens.query)
    }

    async fn query_via_bus(&self, query: &str) -> Result<Vec<LensItem>, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .post("http://127.0.0.1:8081/api/bus/publish")
            .json(&serde_json::json!({
                "topic":   "lens.query",
                "source":  "fs-lenses",
                "payload": { "query": query }
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("bus returned {}", resp.status()));
        }

        // In a real implementation, we'd collect async responses from services.
        // For now, return empty (services haven't implemented lens.query handling yet).
        Ok(Vec::new())
    }

    /// Demonstration items shown when the bus is not reachable.
    fn demo_items(&self, query: &str) -> Vec<LensItem> {
        let q = query.to_string();
        vec![
            LensItem {
                role:    LensRole::Wiki,
                summary: format!("Wiki: Search results for '{q}'"),
                link:    Some(format!("http://wiki.local/search?q={q}")),
                source:  "outline-wiki".into(),
            },
            LensItem {
                role:    LensRole::Chat,
                summary: format!("Chat: 3 messages mentioning '{q}'"),
                link:    Some("http://chat.local/search".into()),
                source:  "matrix".into(),
            },
            LensItem {
                role:    LensRole::Git,
                summary: format!("Git: 2 repositories matching '{q}'"),
                link:    Some("http://git.local/explore".into()),
                source:  "forgejo".into(),
            },
            LensItem {
                role:    LensRole::Tasks,
                summary: format!("Tasks: 5 open tasks for '{q}'"),
                link:    Some("http://tasks.local/".into()),
                source:  "vikunja".into(),
            },
        ]
    }
}

// ── Public shim ───────────────────────────────────────────────────────────────

pub async fn refresh_lens(lens: &Lens) -> Vec<LensItem> {
    LensQueryEngine.refresh_lens(lens).await
}
