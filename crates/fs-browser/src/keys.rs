// keys.rs — FTL key name constants for fs-browser.
//
// All user-visible strings are translated via fs-i18n.
// The matching .ftl files live at:
//   fs-i18n/locales/{lang}/browser.ftl
//
// Use these constants wherever a localised string is needed:
//   fs_i18n::t(keys::TITLE).to_string()

// ── Window ────────────────────────────────────────────────────────────────────

pub const TITLE: &str = "browser-title";
pub const ADDRESS_PLACEHOLDER: &str = "browser-address-placeholder";
pub const NEW_TAB: &str = "browser-new-tab";

// ── Navigation ────────────────────────────────────────────────────────────────

pub const BTN_BACK: &str = "browser-btn-back";
pub const BTN_FORWARD: &str = "browser-btn-forward";
pub const BTN_REFRESH: &str = "browser-btn-refresh";
pub const BTN_HOME: &str = "browser-btn-home";

// ── Tabs ──────────────────────────────────────────────────────────────────────

pub const TAB_HISTORY: &str = "browser-tab-history";
pub const TAB_BOOKMARKS: &str = "browser-tab-bookmarks";
pub const TAB_DOWNLOADS: &str = "browser-tab-downloads";

// ── Bookmarks ─────────────────────────────────────────────────────────────────

pub const BOOKMARKS_TITLE: &str = "browser-bookmarks-title";
pub const BOOKMARKS_ADDED: &str = "browser-bookmarks-added";
pub const BOOKMARKS_EMPTY: &str = "browser-bookmarks-empty";

// ── History ───────────────────────────────────────────────────────────────────

pub const HISTORY_TITLE: &str = "browser-history-title";
pub const HISTORY_EMPTY: &str = "browser-history-empty";
pub const HISTORY_CLEAR: &str = "browser-history-clear";

// ── Errors ────────────────────────────────────────────────────────────────────

pub const ERROR_NO_GUI: &str = "browser-error-no-gui";
