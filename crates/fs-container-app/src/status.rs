/// Display trait for `UnitActiveState` — single source of truth for status
/// label, foreground color, and background color used across all views.
///
/// Implemented as an extension trait because `UnitActiveState` is defined in
/// the external `fs_container` crate and cannot be modified directly.
use fs_container::UnitActiveState;
use fs_i18n;

pub trait UnitActiveStateDisplay {
    /// Localized human-readable status label (e.g. "Running", "Stopped").
    fn status_label(&self) -> String;
    /// CSS color variable for the status foreground.
    fn status_color(&self) -> &'static str;
    /// CSS color variable (or literal) for the status badge background.
    fn status_bg(&self) -> &'static str;
}

impl UnitActiveStateDisplay for UnitActiveState {
    fn status_label(&self) -> String {
        match self {
            UnitActiveState::Active       => fs_i18n::t("status.running").to_string(),
            UnitActiveState::Inactive     => fs_i18n::t("status.stopped").to_string(),
            UnitActiveState::Activating   => fs_i18n::t("status.starting").to_string(),
            UnitActiveState::Deactivating => fs_i18n::t("status.stopping").to_string(),
            UnitActiveState::Failed       => fs_i18n::t("status.failed").to_string(),
            UnitActiveState::Unknown      => fs_i18n::t("status.unknown").to_string(),
        }
    }

    fn status_color(&self) -> &'static str {
        match self {
            UnitActiveState::Active       => "var(--fs-success)",
            UnitActiveState::Inactive     => "var(--fs-text-muted)",
            UnitActiveState::Activating   => "var(--fs-info)",
            UnitActiveState::Deactivating => "var(--fs-warning)",
            UnitActiveState::Failed       => "var(--fs-error)",
            UnitActiveState::Unknown      => "var(--fs-text-muted)",
        }
    }

    fn status_bg(&self) -> &'static str {
        match self {
            UnitActiveState::Active       => "rgba(34,197,94,0.1)",
            UnitActiveState::Inactive     => "var(--fs-bg-elevated)",
            UnitActiveState::Activating   => "rgba(99,179,237,0.1)",
            UnitActiveState::Deactivating => "rgba(251,191,36,0.1)",
            UnitActiveState::Failed       => "rgba(239,68,68,0.1)",
            UnitActiveState::Unknown      => "var(--fs-bg-elevated)",
        }
    }
}
