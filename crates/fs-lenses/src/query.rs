// query.rs — Lens query engine.
//
// Queries the bus for data matching a lens's search term.
// Current implementation returns demo items; real bus integration
// will be wired when fs-bus gRPC client is available.

use crate::model::{Lens, LensItem, LensRole};

// ── LensQueryEngine ───────────────────────────────────────────────────────────

pub struct LensQueryEngine;

impl LensQueryEngine {
    /// Return demo items for a lens query.
    ///
    /// Real implementation will publish to the bus and collect responses.
    pub fn refresh_lens(&self, lens: &Lens) -> Vec<LensItem> {
        Self::demo_items(&lens.query)
    }

    /// Demonstration items shown before real bus routing is wired up.
    fn demo_items(query: &str) -> Vec<LensItem> {
        let q = query.to_string();
        vec![
            LensItem {
                role: LensRole::Wiki,
                summary: format!("Wiki: Search results for '{q}'"),
                link: Some(format!("http://wiki.local/search?q={q}")),
                source: "outline-wiki".into(),
            },
            LensItem {
                role: LensRole::Chat,
                summary: format!("Chat: 3 messages mentioning '{q}'"),
                link: Some("http://chat.local/search".into()),
                source: "matrix".into(),
            },
            LensItem {
                role: LensRole::Git,
                summary: format!("Git: 2 repositories matching '{q}'"),
                link: Some("http://git.local/explore".into()),
                source: "forgejo".into(),
            },
            LensItem {
                role: LensRole::Tasks,
                summary: format!("Tasks: 5 open tasks for '{q}'"),
                link: Some("http://tasks.local/".into()),
                source: "vikunja".into(),
            },
        ]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Lens;

    #[test]
    fn refresh_returns_demo_items() {
        let engine = LensQueryEngine;
        let lens = Lens::new("test", "rust");
        let items = engine.refresh_lens(&lens);
        assert!(!items.is_empty());
    }

    #[test]
    fn demo_items_include_all_roles() {
        let items = LensQueryEngine::demo_items("alpha");
        let roles: Vec<_> = items.iter().map(|i| i.role.id()).collect();
        assert!(roles.contains(&"wiki".to_string()));
        assert!(roles.contains(&"chat".to_string()));
        assert!(roles.contains(&"git".to_string()));
        assert!(roles.contains(&"tasks".to_string()));
    }
}
