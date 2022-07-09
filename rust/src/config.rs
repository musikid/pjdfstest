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

#[derive(Debug, Deserialize)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
}
