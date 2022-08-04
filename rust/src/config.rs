use std::collections::HashMap;

use crate::test::FileFlags;
use crate::test::FileSystemFeature;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct CommonFeatureConfig {}

/// Configuration for file-system specific features.
/// Please see the book for more details.
#[derive(Debug, Default, Deserialize)]
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
/// The user and the group have the same name and the user should be part of the associated group.
#[derive(Debug, Deserialize)]
pub struct DummyAuthConfig {
    pub entries: [String; 3],
}

impl Default for DummyAuthConfig {
    fn default() -> Self {
        Self {
            entries: [
                String::from("nobody"),
                String::from("tests"),
                String::from("dummy"),
            ],
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// File-system features.
    pub features: FeaturesConfig,
    pub settings: SettingsConfig,
    pub dummy_auth: DummyAuthConfig,
}
