/// Domain contexts — centralise state and business logic, keep it out of UI components.
use dioxus::prelude::*;

use crate::model::{
    BotKind, CachedRoom, GroupsConfig, MessagingBot, MessagingBotsConfig,
    RoomCollection, TomlConfig,
};

// ── BotManagerContext ──────────────────────────────────────────────────────────

/// Central context for the Bot Manager.
/// Holds reactive state and provides domain operations with automatic persistence.
/// Provide once in `BotManagerApp` via `provide_context`; read in child components
/// via `use_context::<BotManagerContext>()`.
#[derive(Clone)]
pub struct BotManagerContext {
    pub bots:         Signal<Vec<MessagingBot>>,
    pub selected_idx: Signal<Option<usize>>,
}

impl BotManagerContext {
    /// Update a bot by index and persist the full list.
    pub fn update_bot(&self, idx: usize, updated: MessagingBot) {
        // Signal<T>: Copy — copy to local to get mutable access via interior mutability
        let mut bots = self.bots;
        bots.write()[idx] = updated;
        let _ = MessagingBotsConfig { bots: bots.read().clone() }.save();
    }

    /// Update whichever bot is currently selected.
    pub fn update_selected(&self, updated: MessagingBot) {
        if let Some(i) = *self.selected_idx.read() {
            self.update_bot(i, updated);
        }
    }

    /// Return the first bot of the given kind together with its index.
    pub fn bot_by_kind(&self, kind: &BotKind) -> Option<(usize, MessagingBot)> {
        self.bots.read().iter().enumerate()
            .find(|(_, b)| &b.kind == kind)
            .map(|(i, b)| (i, b.clone()))
    }
}

// ── GroupsContext ──────────────────────────────────────────────────────────────

/// Context for the Groups view — owns collections and rooms state.
/// Provide in `GroupsView` via `provide_context`; mutations trigger automatic save.
#[derive(Clone)]
pub struct GroupsContext {
    pub collections:    Signal<Vec<RoomCollection>>,
    pub rooms:          Signal<Vec<CachedRoom>>,
    pub sel_collection: Signal<Option<u32>>,
}

impl GroupsContext {
    fn persist(&self) {
        let cfg = GroupsConfig {
            collections:  self.collections.read().clone(),
            cached_rooms: self.rooms.read().clone(),
        };
        let _ = cfg.save();
    }

    /// Add a new collection. No-op if `name` is blank.
    pub fn add_collection(&self, name: String, description: String) {
        if name.trim().is_empty() { return; }
        let next_id = self.collections.read().iter().map(|c| c.id).max().unwrap_or(0) + 1;
        let mut collections = self.collections;
        collections.write().push(RoomCollection {
            id:          next_id,
            name:        name.trim().to_string(),
            description: description.trim().to_string(),
            members:     vec![],
        });
        self.persist();
    }

    /// Add rooms to a collection, skipping duplicates.
    pub fn add_rooms_to_collection(&self, col_id: u32, rooms: Vec<(String, String)>) {
        let mut collections = self.collections;
        if let Some(c) = collections.write().iter_mut().find(|c| c.id == col_id) {
            for room_ref in rooms {
                if !c.members.contains(&room_ref) { c.members.push(room_ref); }
            }
        }
        self.persist();
    }

    /// Remove selected rooms from a collection.
    pub fn remove_rooms_from_collection(&self, col_id: u32, rooms: Vec<(String, String)>) {
        let mut collections = self.collections;
        if let Some(c) = collections.write().iter_mut().find(|c| c.id == col_id) {
            c.members.retain(|m| !rooms.contains(m));
        }
        self.persist();
    }
}
