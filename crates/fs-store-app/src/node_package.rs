/// NodePackage — local domain type for display and install logic.
///
/// Built from an `Arc<dyn fs_store::Package>` when loaded from the Store catalog.
pub use fs_db_desktop::package_registry::PackageKind;

use std::sync::Arc;

use fs_store::Package;
use serde::{Deserialize, Serialize};

use crate::browser::resolve_icon_path;

/// A package entry ready for display and install.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub kind: PackageKind,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub icon: Option<String>,
    /// Store-relative path to the package directory (used for downloads).
    #[serde(default)]
    pub path: Option<String>,
}

impl NodePackage {
    /// Build a `NodePackage` from a store catalog `Arc<dyn Package>`.
    pub fn from_package(pkg: &Arc<dyn Package>) -> Self {
        let category_id = pkg.category().id();
        let icon = pkg.icon_path().and_then(resolve_icon_path);

        Self {
            id: pkg.id().to_owned(),
            name: pkg.name().to_owned(),
            version: pkg.latest_version().unwrap_or("0.0.0").to_owned(),
            category: category_id.to_owned(),
            description: pkg.description().to_owned(),
            license: String::new(),
            author: String::new(),
            tags: pkg.tags().to_vec(),
            kind: PackageKind::from_category(category_id),
            capabilities: vec![],
            icon,
            path: Some(pkg.help().base_path.clone()),
        }
    }
}

/// Extension on `PackageKind` to convert from Store category IDs.
pub trait PackageKindExt {
    fn from_category(category_id: &str) -> Self;
}

impl PackageKindExt for PackageKind {
    fn from_category(category_id: &str) -> Self {
        match category_id {
            "containers" => Self::Container,
            "bundles" => Self::Bundle,
            "themes" => Self::Theme,
            "widgets" => Self::Widget,
            "languages" => Self::Language,
            "tasks" => Self::Task,
            _ => Self::App,
        }
    }
}
