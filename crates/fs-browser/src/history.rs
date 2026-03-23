// Browser history helpers.

use crate::model::HistoryEntry;

/// Remove all history entries.
pub fn clear(history: &mut Vec<HistoryEntry>) {
    history.clear();
}

/// Return the last N entries (most recent first).
pub fn recent(history: &[HistoryEntry], limit: usize) -> Vec<&HistoryEntry> {
    history.iter().rev().take(limit).collect()
}
