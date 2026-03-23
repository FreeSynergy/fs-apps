/// NodePackage — package type for the FreeSynergy Node.Store catalog.
pub use fs_db_desktop::package_registry::PackageKind;
use fs_store::manifest::Manifest;
use serde::{Deserialize, Serialize};

/// A package entry in the `Node/catalog.toml`.
///
/// Extends `PackageMeta` with Node-Store-specific fields (`icon`, `path`, `kind`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePackage {
    pub id:          String,
    pub name:        String,
    pub version:     String,
    pub category:    String,
    pub description: String,
    #[serde(default)]
    pub license:     String,
    #[serde(default)]
    pub author:      String,
    #[serde(default)]
    pub tags:        Vec<String>,
    #[serde(default)]
    pub kind:         PackageKind,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub icon:         Option<String>,
    /// Store-relative path to the module directory.
    #[serde(default)]
    pub path:         Option<String>,
}

impl Manifest for NodePackage {
    fn id(&self)       -> &str { &self.id }
    fn version(&self)  -> &str { &self.version }
    fn category(&self) -> &str { &self.category }
    fn name(&self)     -> &str { &self.name }
}
