// controller.rs — BotController: facade over MessagingBotsConfig.

use std::sync::{Arc, Mutex};

use crate::model::{MessagingBot, MessagingBotsConfig};

/// Shared, cheaply-clonable controller for bot operations.
#[derive(Clone)]
pub struct BotController {
    bots: Arc<Mutex<Vec<MessagingBot>>>,
}

impl BotController {
    #[must_use]
    pub fn new() -> Self {
        let bots = MessagingBotsConfig::load();
        Self {
            bots: Arc::new(Mutex::new(bots)),
        }
    }

    pub fn list(&self) -> Vec<MessagingBot> {
        self.bots.lock().unwrap().clone()
    }

    pub fn get(&self, id: &str) -> Option<MessagingBot> {
        self.bots
            .lock()
            .unwrap()
            .iter()
            .find(|b| b.id == id)
            .cloned()
    }

    /// Enable a bot. Returns `true` if found, `false` if not found.
    pub fn enable(&self, id: &str) -> bool {
        let mut guard = self.bots.lock().unwrap();
        if let Some(bot) = guard.iter_mut().find(|b| b.id == id) {
            bot.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a bot. Returns `true` if found, `false` if not found.
    pub fn disable(&self, id: &str) -> bool {
        let mut guard = self.bots.lock().unwrap();
        if let Some(bot) = guard.iter_mut().find(|b| b.id == id) {
            bot.enabled = false;
            true
        } else {
            false
        }
    }
}

impl Default for BotController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_controller_loads_bots() {
        let ctrl = BotController::new();
        // Demo bots are returned when config is empty.
        assert!(!ctrl.list().is_empty());
    }

    #[test]
    fn enable_disable_known_bot() {
        let ctrl = BotController::new();
        let bots = ctrl.list();
        let id = bots[0].id.clone();
        assert!(ctrl.disable(&id));
        assert!(!ctrl.get(&id).unwrap().enabled);
        assert!(ctrl.enable(&id));
        assert!(ctrl.get(&id).unwrap().enabled);
    }

    #[test]
    fn unknown_bot_returns_false() {
        let ctrl = BotController::new();
        assert!(!ctrl.enable("nonexistent-id"));
        assert!(!ctrl.disable("nonexistent-id"));
    }
}
