use std::fmt::Debug;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
pub use crate::runner::context::{SerializedTestContext, TestContext};

/// Error returned by a test function.
#[derive(Error, Debug)]
pub enum TestError {
    #[error("error while calling syscall: {0}")]
    Nix(#[from] nix::Error),
}

/// Function which indicates if the test should be skipped by returning an error.
pub type Guard = fn(&Config, &Path) -> Result<(), anyhow::Error>;

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

#[allow(non_camel_case_types)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    Serialize,
    Deserialize,
)]
/// File flags (see https://docs.freebsd.org/en/books/handbook/basics/#permissions).
pub enum FileFlags {
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_SETTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_NODUMP,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_IMMUTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_APPEND,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_OPAQUE,

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_SETTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_ARCHIVED,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_IMMUTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_APPEND,

    #[cfg(any(target_os = "dragonfly"))]
    UF_NOHISTORY,
    #[cfg(any(target_os = "dragonfly"))]
    UF_CACHE,
    #[cfg(any(target_os = "dragonfly"))]
    UF_XLINK,
    #[cfg(any(target_os = "dragonfly"))]
    SF_NOHISTORY,
    #[cfg(any(target_os = "dragonfly"))]
    SF_CACHE,
    #[cfg(any(target_os = "dragonfly"))]
    SF_XLINK,

    #[cfg(any(target_os = "freebsd"))]
    UF_SYSTEM,
    #[cfg(any(target_os = "freebsd"))]
    UF_SPARSE,
    #[cfg(any(target_os = "freebsd"))]
    UF_OFFLINE,
    #[cfg(any(target_os = "freebsd"))]
    UF_REPARSE,
    #[cfg(any(target_os = "freebsd"))]
    UF_ARCHIVE,
    #[cfg(any(target_os = "freebsd"))]
    UF_READONLY,

    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    SF_SNAPSHOT,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    UF_NOUNLINK,
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    SF_NOUNLINK,

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
    UF_COMPRESSED,
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
    UF_TRACKED,

    #[cfg(any(
        target_os = "freebsd",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos"
    ))]
    UF_HIDDEN,

    #[cfg(any(target_os = "netbsd"))]
    SF_LOG,
    #[cfg(any(target_os = "netbsd"))]
    SF_SNAPINVAL,
}

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
