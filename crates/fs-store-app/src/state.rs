/// Shared reactive state for fs-store.
///
/// INSTALL_COUNTER is incremented whenever a package is installed or removed.
/// Any component that reads this signal will automatically re-run when it changes.
use dioxus::prelude::*;

/// Bump this after any install or remove to trigger reactive refreshes.
pub static INSTALL_COUNTER: GlobalSignal<u32> = Signal::global(|| 0);

/// Increment the install counter — call after install or remove.
pub fn notify_install_changed() {
    *INSTALL_COUNTER.write() += 1;
}
