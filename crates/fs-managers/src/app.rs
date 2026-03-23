/// ManagersApp — launches the standardized Package Manager window.
///
/// In standalone mode: shows a placeholder.
/// When embedded in the Desktop: ManagerView is called directly with real package data.
use dioxus::prelude::*;

/// Root app component for the standalone Manager binary.
#[component]
pub fn ManagersApp() -> Element {
    rsx! {
        div {
            style: "height: 100vh; display: flex; align-items: center; justify-content: center; \
                    background: var(--fs-bg-base, #0c1222); color: var(--fs-text-muted, #5a6e88); \
                    font-family: system-ui, sans-serif; font-size: 14px;",
            "Open a package from the Store or the Desktop to manage it here."
        }
    }
}
