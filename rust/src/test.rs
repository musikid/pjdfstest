use std::path::Path;

use crate::config::Config;
pub use crate::context::{SerializedTestContext, TestContext};
pub use crate::features::*;
pub use crate::flags::*;

/// Function which indicates if the test should be skipped by returning an error.
pub type Guard = fn(&Config, &Path) -> Result<(), anyhow::Error>;

#[derive(Clone, Copy)]
pub enum TestFn {
    Serialized(fn(&mut SerializedTestContext)),
    NonSerialized(fn(&mut TestContext)),
}

/// A single minimal test case.
pub struct TestCase {
    pub name: &'static str,
    pub description: &'static str,
    pub require_root: bool,
    pub fun: TestFn,
    pub required_features: &'static [FileSystemFeature],
    pub guards: &'static [Guard],
}

inventory::collect!(TestCase);
