use std::fmt::Debug;

use linkme::distributed_slice;
use serde::Deserialize;
use thiserror::Error;

use crate::runner::context::ContextError;
pub use crate::runner::context::TestContext;

pub type TestResult = std::result::Result<(), TestError>;

/// Error returned by a test function.
#[derive(Error, Debug)]
pub enum TestError {
    #[error("error while creating file: {0}")]
    CreateFile(ContextError),
    #[error("error while calling syscall: {0}")]
    Nix(#[from] nix::Error),
}

/// A single minimal test case.
pub struct TestCase {
    pub name: &'static str,
    pub require_root: bool,
    pub fun: fn(&mut TestContext),
    pub required_features: Option<&'static [FileSystemFeature]>,
}

#[distributed_slice]
pub static TEST_CASES: [TestCase] = [..];

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
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
    Deserialize,
)]
/// File flags (see https://docs.freebsd.org/en/books/handbook/basics/#permissions).
pub enum FileFlags {
    UF_SETTABLE,
    UF_NODUMP,
    UF_IMMUTABLE,
    UF_APPEND,
    UF_OPAQUE,

    SF_SETTABLE,
    SF_ARCHIVED,
    SF_IMMUTABLE,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumIter, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FileSystemFeature {
    Chflags,
    ChflagsSfSnapshot,
    #[strum(disabled)]
    #[serde(skip)]
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    //TODO: Create another structure for flags? or directly add them into this enum?
    FileFlags(&'static [FileFlags]),
    PosixFallocate,
    RenameCtime,
    StatStBirthtime,
    UtimeNow,
}
