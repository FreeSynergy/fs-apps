// package_type_display.rs — Display extension trait for the external PackageType.
//
// `PackageType` is defined in `fs_pkg::manifest` (an external crate). We cannot
// add methods to it directly, so we use an extension trait to attach UI-specific
// behaviour (human-readable label + CSS modifier class) without touching fs-pkg.

use fs_pkg::manifest::PackageType;

/// Extension trait that provides UI display information for [`PackageType`].
///
/// Implemented for the external `PackageType` enum so that view code can call
/// `pkg_type.display_label()` and `pkg_type.type_css()` directly instead of
/// delegating to a private helper on `PackageViewModel`.
pub trait PackageTypeDisplay {
    /// Human-readable, title-case label suitable for display in the UI.
    fn display_label(&self) -> &'static str;

    /// CSS BEM modifier class for styling the package type badge.
    fn type_css(&self) -> &'static str;
}

impl PackageTypeDisplay for PackageType {
    fn display_label(&self) -> &'static str {
        match self {
            PackageType::App       => "App",
            PackageType::Container => "Container",
            PackageType::Bundle    => "Bundle",
            PackageType::Language  => "Language",
            PackageType::Theme     => "Theme",
            PackageType::Widget    => "Widget",
            PackageType::Bot       => "Bot",
            PackageType::Bridge    => "Bridge",
            PackageType::Task      => "Task",
        }
    }

    fn type_css(&self) -> &'static str {
        match self {
            PackageType::App       => "fs-type--app",
            PackageType::Container => "fs-type--container",
            PackageType::Bundle    => "fs-type--bundle",
            PackageType::Language  => "fs-type--language",
            PackageType::Theme     => "fs-type--theme",
            PackageType::Widget    => "fs-type--widget",
            PackageType::Bot       => "fs-type--bot",
            PackageType::Bridge    => "fs-type--bridge",
            PackageType::Task      => "fs-type--task",
        }
    }
}
