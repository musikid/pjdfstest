//! File-system features which are not available on every file system and can be tested for.
//!
//! This module defines an enum which represents features which are not available on every file system,
//! but can be tested for. The features are used to define which tests should be run on which file systems.

use serde::{Deserialize, Serialize};

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
    /// The [`chflags`](https://man.freebsd.org/cgi/man.cgi?chflags(1)) command is available
    Chflags,
    /// NFSv4 style Access Control Lists are available
    Nfsv4Acls,
    /// The [`posix_fallocate`](https://pubs.opengroup.org/onlinepubs/007904975/functions/posix_fallocate.html) syscall is available
    PosixFallocate,
    /// [`rename`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/rename.html) changes `st_ctime` on success (POSIX does not require a file system to update a file's ctime when it gets renamed, but some file systems choose to do it anyway)
    RenameCtime,
    /// `struct stat` contains an [`st_birthtime`](https://man.freebsd.org/cgi/man.cgi?stat(2)) field
    StatStBirthtime,
    /// The [`SF_SNAPSHOT`](https://man.freebsd.org/cgi/man.cgi?chflags(2)) flag can be set with `chflags`
    ChflagsSfSnapshot,
    /// The [`UTIME_NOW`](https://pubs.opengroup.org/onlinepubs/9699919799.orig/functions/futimens.html) constant is available
    UtimeNow,
    /// The [`utimensat`](https://pubs.opengroup.org/onlinepubs/9699919799.orig/functions/utimensat.html) syscall is available
    Utimensat,
}
