use std::collections::HashMap;

use pjdfs_tests::test::FileFlags;
use pjdfs_tests::test::FileSystemFeature;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommonFeatureConfig {}

/// Configuration for file-system specific features.
/// Please see the book for more details.
#[derive(Debug, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default)]
    pub file_flags: Vec<FileFlags>,
    #[serde(flatten)]
    pub fs_features: HashMap<FileSystemFeature, CommonFeatureConfig>,
}

/// Adjustable file-system specific settings.
/// Please see the book for more details.
#[derive(Debug, Deserialize)]
pub struct SettingsConfig {
    #[serde(default = "default_naptime")]
    pub naptime: f64
}

fn default_naptime() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
    pub settings: SettingsConfig
}
