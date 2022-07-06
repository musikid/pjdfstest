use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FeatureConfig {}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Opt-in syscalls.
    pub features: HashMap<String, FeatureConfig>,
}
