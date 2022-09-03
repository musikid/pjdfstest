use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::features::FileSystemFeature;
use crate::flags::FileFlags;
use figment::value::Value;
use nix::unistd::Group;
use nix::unistd::User;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommonFeatureConfig {}

/// Configuration for file-system specific features.
/// Please see the book for more details.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default)]
    pub file_flags: HashSet<FileFlags>,
    pub eperm: EpermConfig,
    #[serde(flatten)]
    pub fs_features: HashMap<FileSystemFeature, CommonFeatureConfig>,
}

/// Adjustable file-system specific settings.
/// Please see the book for more details.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsConfig {
    #[serde(default = "default_naptime")]
    pub naptime: f64,
    #[serde(default)]
    pub erofs: bool,
    pub secondary_fs: Option<PathBuf>,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        SettingsConfig {
            naptime: default_naptime(),
            erofs: false,
            secondary_fs: None,
        }
    }
}

const fn default_naptime() -> f64 {
    1.0
}

/// Auth entries, which are composed of a [`User`](nix::unistd::User) and its associated [`Group`](nix::unistd::Group).
/// The user should be part of the associated group.
#[derive(Debug, Serialize, Deserialize)]
pub struct DummyAuthConfig {
    pub entries: [(String, String); 3],
}

impl Default for DummyAuthConfig {
    fn default() -> Self {
        Self {
            entries: [
                (
                    String::from("nobody"),
                    Group::from_gid(User::from_name("nobody").unwrap().unwrap().gid)
                        .unwrap()
                        .unwrap()
                        .name,
                ),
                (String::from("tests"), String::from("tests")),
                (String::from("pjdfstest"), String::from("pjdfstest")),
            ],
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EpermConfig {
    pub syscalls_flags: HashMap<String, Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
    pub settings: SettingsConfig,
    pub dummy_auth: DummyAuthConfig,
}

/// Return flags which intersects with the provided ones
/// and those available in the configuration,
/// along with the other available in the configuration.
pub fn get_flags_intersection(
    config: &FeaturesConfig,
    flags: &[FileFlags],
) -> (Vec<FileFlags>, Vec<FileFlags>) {
    let flags: HashSet<_> = flags.iter().copied().collect();
    let eperm_flags: HashSet<_> = config.file_flags.intersection(&flags).copied().collect();
    let valid_flags: Vec<_> = config
        .file_flags
        .difference(&eperm_flags)
        .copied()
        .collect();

    (eperm_flags.into_iter().collect(), valid_flags)
}
