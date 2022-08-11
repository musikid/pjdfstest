use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use crate::flags::*;
pub use crate::runner::context::{SerializedTestContext, TestContext};

/// Error returned by a test function.
#[derive(Error, Debug)]
pub enum TestError {
    #[error("error while calling syscall: {0}")]
    Nix(#[from] nix::Error),
}

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
    pub required_file_flags: &'static [FileFlags],
}

inventory::collect!(TestCase);

/// Features which are not available for every file system.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumIter,
    strum::EnumMessage,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FileSystemFeature {
    /// The chflags syscall is available
    Chflags,
    /// The posix_fallocate syscall is available
    PosixFallocate,
    /// rename changes st_ctime on success
    /// POSIX does not require a file system to update a file's ctime when it gets renamed, but some file systems choose to do it anyway.
    RenameCtime,
    /// struct stat contains an st_birthtime field
    StatStBirthtime,
    /// The UTIME_NOW constant is available
    UtimeNow,
    /// The utimensat syscall is available
    Utimensat,
}
