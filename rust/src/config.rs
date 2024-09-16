//! Configuration for the test suite.
//!
//! This module contains the configuration for the test suite. It is used to configure the test suite
//! and to define which tests should be run on which file systems.
//!
//! The configuration is loaded from a TOML file, which is passed as a command line argument to the test suite.

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::test::FileFlags;
use crate::test::FileSystemFeature;
use serde::{Deserialize, Serialize};

mod auth;
pub use auth::*;

/// Configuration for dummy authentication.
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
    // TODO: Move to another part of the configuration when refactoring
    #[serde(default)]
    pub secondary_fs: Option<PathBuf>,
    /// File-system specific features which are enabled
    /// and do not require any additional configuration.
    #[serde(flatten)]
    pub fs_features: HashMap<FileSystemFeature, CommonFeatureConfig>,
}

/// Adjustable file-system specific settings.
/// Please see the book for more details.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// Time to sleep within tests (in milliseconds)
    /// between modifications to the file system.
    /// It should be set to a value that is at least greater than
    /// the timestamp granularity of the file system under test.
    #[serde(default = "default_naptime")]
    pub naptime: f64,
    /// Allow remounting the file system with different settings during tests
    /// (required for example by the `erofs` tests).
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
