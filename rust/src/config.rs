use std::collections::HashMap;

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
use pjdfs_tests::test::FileFlags;
use pjdfs_tests::test::FileSystemFeature;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommonFeatureConfig {}

/// Configuration for file-system specific features.
/// Please see the book for more details.
#[derive(Debug, Deserialize)]
pub struct FeaturesConfig {
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    pub file_flags: Option<Vec<FileFlags>>,
    #[serde(flatten)]
    pub fs_features: HashMap<FileSystemFeature, CommonFeatureConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
}
