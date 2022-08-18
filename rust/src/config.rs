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
}

impl Default for SettingsConfig {
    fn default() -> Self {
        SettingsConfig {
            naptime: default_naptime(),
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
pub struct ErofsConfig {
    pub enabled: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExdevConfig {
    pub secondary_fs: Option<PathBuf>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
    pub settings: SettingsConfig,
    pub dummy_auth: DummyAuthConfig,
}
