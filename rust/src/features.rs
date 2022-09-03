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
    /// The chflags syscall is available
    Chflags,
    /// The posix_fallocate syscall is available
    PosixFallocate,
    /// rename changes st_ctime on success
    /// POSIX does not require a file system to update a file's ctime when it gets renamed, but some file systems choose to do it anyway.
    RenameCtime,
    /// struct stat contains an st_birthtime field
    StatStBirthtime,
    /// The SF_SNAPSHOT flag can be set with chflags
    ChflagsSfSnapshot,
    /// The UTIME_NOW constant is available
    UtimeNow,
    /// The utimensat syscall is available
    Utimensat,
}
