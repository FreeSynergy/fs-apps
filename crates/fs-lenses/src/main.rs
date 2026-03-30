#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
fn main() {
    #[cfg(feature = "desktop")]
    fs_components::launch_desktop(
        fs_components::DesktopConfig::new()
            .with_title("FSN Lenses")
            .with_size(1000.0, 700.0),
        fs_lenses::LensesApp,
    );
}
