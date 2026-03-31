// controller.rs — ThemeController: MVC controller for the theme manager.
//
// Knows only the ThemeLoader trait — never touches fs-theme directly.
// Each user action is a Command that can be applied to the registry.

use std::sync::{Arc, Mutex};

use fs_theme::ThemeRegistry;

// ── ThemeInfo ─────────────────────────────────────────────────────────────────

/// Lightweight theme descriptor returned by the controller.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ThemeInfo {
    /// Theme name (unique identifier).
    pub name: String,
    /// Theme version string.
    pub version: String,
}

// ── ThemeController ───────────────────────────────────────────────────────────

/// MVC controller — owns the shared `ThemeRegistry`.
///
/// All mutating operations go through this controller, not the registry directly.
#[derive(Clone)]
pub struct ThemeController {
    registry: Arc<Mutex<ThemeRegistry>>,
}

impl ThemeController {
    /// Create a controller with the default built-in themes.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ThemeRegistry::default())),
        }
    }

    /// List all available themes.
    pub fn list(&self) -> Vec<ThemeInfo> {
        let reg = self.registry.lock().unwrap();
        let ver = reg.active().version.clone();
        reg.names()
            .into_iter()
            .map(|name| ThemeInfo {
                name: name.to_owned(),
                version: ver.clone(),
            })
            .collect()
    }

    /// Return the currently active theme.
    pub fn active(&self) -> ThemeInfo {
        let reg = self.registry.lock().unwrap();
        let t = reg.active();
        ThemeInfo {
            name: t.name.clone(),
            version: t.version.clone(),
        }
    }

    /// Activate a theme by name. Returns `Err` if the name is unknown.
    pub fn activate(&self, name: &str) -> Result<(), String> {
        let mut reg = self.registry.lock().unwrap();
        reg.set_active(name).map_err(|e| e.to_string())
    }

    /// Return the CSS for a named theme (for preview).
    ///
    /// Temporarily activates the theme to read its CSS, then restores the original.
    pub fn preview_css(&self, name: &str) -> Option<String> {
        let mut reg = self.registry.lock().unwrap();
        let original = reg.active().name.clone();
        reg.set_active(name).ok()?;
        let css = reg.active_engine().to_css();
        let _ = reg.set_active(&original);
        Some(css)
    }
}

impl Default for ThemeController {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_returns_built_in_themes() {
        let ctrl = ThemeController::new();
        let themes = ctrl.list();
        assert!(
            !themes.is_empty(),
            "registry should contain at least one theme"
        );
    }

    #[test]
    fn active_returns_default() {
        let ctrl = ThemeController::new();
        let active = ctrl.active();
        assert!(!active.name.is_empty());
    }

    #[test]
    fn activate_unknown_returns_err() {
        let ctrl = ThemeController::new();
        assert!(ctrl.activate("__does_not_exist__").is_err());
    }
}
