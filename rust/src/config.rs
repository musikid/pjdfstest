use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::test::FileFlags;
use crate::test::FileSystemFeature;
use serde::{Deserialize, Serialize};

mod auth;
pub use auth::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommonFeatureConfig {}

/// Configuration for file-system specific features.
/// Please see the book for more details.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// File flags available in the file system.
    #[serde(default)]
    pub file_flags: HashSet<FileFlags>,
    /// Secondary file system to use for cross-file-system tests.
    // TODO: Move to a separate struct when suite is refactored.
    #[serde(default)]
    pub secondary_fs: Option<PathBuf>,
    /// File-system specific features which are enabled and do
    #[serde(flatten)]
    pub fs_features: HashMap<FileSystemFeature, CommonFeatureConfig>,
}

/// Adjustable file-system specific settings.
/// Please see the book for more details.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// Time to sleep between tests.
    #[serde(default = "default_naptime")]
    pub naptime: f64,
    /// Allow remounting the file system with different settings during tests (required for example by the `erofs` test).
    pub allow_remount: bool,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        SettingsConfig {
            naptime: default_naptime(),
            allow_remount: false,
        }
    }
}

const fn default_naptime() -> f64 {
    1.0
}

/// Configuration for the test suite.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
    /// File-system specific settings.
    pub settings: SettingsConfig,
    /// Dummy authentication configuration.
    pub dummy_auth: DummyAuthConfig,
}
