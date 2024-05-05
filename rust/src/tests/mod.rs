use std::fs::symlink_metadata;
use std::ops::{BitAnd, BitOr};
use std::os::unix::fs::MetadataExt as StdMetadataExt;

use std::{fs::metadata, path::Path};

#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use nix::sys::stat::stat;
use nix::sys::time::TimeSpec;

use crate::test::TestContext;

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
pub mod chflags;
pub mod chmod;
pub mod chown;
pub mod errors;
pub mod ftruncate;
pub mod link;
pub mod mkdir;
pub mod mkfifo;
pub mod mknod;
mod mksyscalls;
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub mod nfsv4acl;
pub mod open;
pub mod posix_fallocate;
pub mod rename;
pub mod rmdir;
pub mod symlink;
pub mod truncate;
pub mod unlink;
pub mod utimensat;

/// Argument to set which fields should be compared for [`TimeAssertion::path`].
#[derive(Debug, Clone, Copy)]
struct TimestampField(u32);

const ATIME: TimestampField = TimestampField(0b001);
const CTIME: TimestampField = TimestampField(0b010);
const MTIME: TimestampField = TimestampField(0b100);

impl PartialEq<u32> for TimestampField {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl BitAnd for TimestampField {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for TimestampField {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// A handy extention to std::os::unix::fs::MetadataExt
trait MetadataExt: StdMetadataExt {
    /// Return the file's last accessed time as a `TimeSpec`, including
    /// fractional component.
    fn atime_ts(&self) -> TimeSpec {
        TimeSpec::new(self.atime(), self.atime_nsec())
    }

    /// Return the file's last changed time as a `TimeSpec`, including
    /// fractional component.
    fn ctime_ts(&self) -> TimeSpec {
        TimeSpec::new(self.ctime(), self.ctime_nsec())
    }

    /// Return the file's last modified time as a `TimeSpec`, including
    /// fractional component.
    fn mtime_ts(&self) -> TimeSpec {
        TimeSpec::new(self.mtime(), self.mtime_nsec())
    }
}

impl<T: StdMetadataExt> MetadataExt for T {}

/// Metadata which isn't related to time.
#[derive(Debug, PartialEq)]
struct InvariantTimeMetadata {
    st_dev: nix::libc::dev_t,
    st_ino: nix::libc::ino_t,
    st_mode: nix::libc::mode_t,
    st_nlink: nix::libc::nlink_t,
    st_uid: nix::libc::uid_t,
    st_gid: nix::libc::gid_t,
    st_rdev: nix::libc::dev_t,
    st_size: nix::libc::off_t,
    st_blksize: nix::libc::blksize_t,
    st_blocks: nix::libc::blkcnt_t,
}

trait AsTimeInvariant {
    fn as_time_invariant(&self) -> InvariantTimeMetadata;
}

impl AsTimeInvariant for nix::sys::stat::FileStat {
    fn as_time_invariant(&self) -> InvariantTimeMetadata {
        InvariantTimeMetadata {
            st_dev: self.st_dev,
            st_ino: self.st_ino,
            st_mode: self.st_mode,
            st_nlink: self.st_nlink,
            st_uid: self.st_uid,
            st_gid: self.st_gid,
            st_rdev: self.st_rdev,
            st_size: self.st_size,
            st_blksize: self.st_blksize,
            st_blocks: self.st_blocks,
        }
    }
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
// Note: can't be a method of MetadataExt, because StdMetadataExt lacks a
// birthtime() method.
fn birthtime_ts(path: &Path) -> TimeSpec {
    let sb = stat(path).unwrap();
    TimeSpec::new(sb.st_birthtime, sb.st_birthtime_nsec)
}

#[derive(Debug)]
#[must_use]
/// Builder to create a time metadata assertion,
/// which compares metadata between pairs of paths.
struct TimeAssertion<'a> {
    compared_paths: Vec<(&'a Path, &'a Path, TimestampField)>,
    equal: bool,
}

impl<'a> TimeAssertion<'a> {
    /// Return a new builder.
    /// Comparison will be an equality check if `equal` is true, or an ordering one if it is false.
    pub fn new(equal: bool) -> Self {
        Self {
            compared_paths: vec![],
            equal,
        }
    }

    /// Add a path that should compare with itself.
    pub fn path(self, path: &'a Path, fields: TimestampField) -> Self {
        self.paths(path, path, fields)
    }

    /// Add paths that should compare.
    pub fn paths(
        mut self,
        path_before: &'a Path,
        path_after: &'a Path,
        fields: TimestampField,
    ) -> Self {
        self.compared_paths.push((path_before, path_after, fields));
        self
    }

    /// Build the assertion and asserts that `before` metadata
    /// is either equal to or different from the `after` metadata.
    pub fn execute<F>(self, ctx: &TestContext, no_follow_symlink: bool, f: F)
    where
        F: FnOnce(),
    {
        let get_metadata = if no_follow_symlink {
            symlink_metadata
        } else {
            metadata
        };

        let metas_before: Vec<_> = self
            .compared_paths
            .iter()
            .map(|&(path, _, fields)| {
                let meta = get_metadata(path).unwrap();
                (
                    (fields & ATIME != 0).then(|| meta.atime_ts()),
                    (fields & CTIME != 0).then(|| meta.ctime_ts()),
                    (fields & MTIME != 0).then(|| meta.mtime_ts()),
                )
            })
            .collect();

        ctx.nap();

        f();

        let metas_after: Vec<_> = self
            .compared_paths
            .into_iter()
            .map(|(_, path, fields)| {
                let meta = get_metadata(path).unwrap();
                (
                    (fields & ATIME != 0).then(|| meta.atime_ts()),
                    (fields & CTIME != 0).then(|| meta.ctime_ts()),
                    (fields & MTIME != 0).then(|| meta.mtime_ts()),
                )
            })
            .collect();

        if self.equal {
            assert!(
                metas_before
                    .iter()
                    .zip(metas_after.iter())
                    .all(|(mb, ma)| mb == ma),
                "Timestamps changed but shouldn't have"
            );
        } else {
            assert!(
                metas_before
                    .iter()
                    .zip(metas_after.iter())
                    .all(|(mb, ma)| mb != ma),
                "Timestamps did not change as expected"
            );
        }
    }
}

/// Alias for `TimeAssertion::new(false)`.
fn assert_times_changed<'a>() -> TimeAssertion<'a> {
    TimeAssertion::new(false)
}

/// Alias for `TimeAssertion::new(true)`.
fn assert_times_unchanged<'a>() -> TimeAssertion<'a> {
    TimeAssertion::new(true)
}

/// Assert that a certain operation changes the ctime of a file.
fn assert_ctime_changed<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    assert_times_changed()
        .path(path, CTIME)
        .execute(ctx, false, f)
}

/// Assert that a certain operation changes the mtime of a file.
fn assert_mtime_changed<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    assert_times_changed()
        .path(path, MTIME)
        .execute(ctx, false, f)
}

/// Assert that a certain operation does not change the ctime of a file.
fn assert_ctime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    assert_times_unchanged()
        .path(path, CTIME)
        .execute(ctx, false, f)
}

/// Assert that a certain operation does not change the ctime of a file without following symlinks.
fn assert_symlink_ctime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    assert_times_unchanged()
        .path(path, CTIME)
        .execute(ctx, true, f)
}

/// Guard to conditionally skip tests on platforms which do not support
/// the required file flags.
#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
macro_rules! supports_file_flags {
    ($($flags: ident),*) => {
        |config, _| {
            let flags = &[ $(crate::flags::FileFlags::$flags),* ].iter().copied().collect();
            if config.features.file_flags.is_superset(&flags) {
                Ok(())
            } else {
                let unsupported_flags = flags.difference(&config.features.file_flags)
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                anyhow::bail!("file flags {unsupported_flags} aren't supported")
            }
        }
    };
}

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
use supports_file_flags;

#[cfg(test)]
mod t {
    use crate::{config::Config, test::TestCase};

    use super::*;

    crate::test_case! {support_flags_empty; supports_file_flags!()}
    fn support_flags_empty(_: &mut TestContext) {}
    #[test]
    fn support_flags_test_empty() {
        let config = Config::default();
        let tc: &TestCase = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::tests::t::support_flags_empty")
            .unwrap();
        assert_eq!(tc.guards.len(), 1);

        let guard = &tc.guards[0];
        assert!(guard(&config, Path::new("test")).is_ok());
    }

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    crate::test_case! {support_flags_unique; supports_file_flags!(SF_APPEND)}
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    fn support_flags_unique(_: &mut TestContext) {}
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    #[test]
    fn support_flags_test_unique() {
        use std::collections::HashSet;

        use crate::flags::FileFlags;

        let mut config = Config::default();
        let tc: &TestCase = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::tests::t::support_flags_unique")
            .unwrap();
        assert_eq!(tc.guards.len(), 1);

        let guard = &tc.guards[0];
        assert!(guard(&config, Path::new("test")).is_err());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_ok());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND, FileFlags::UF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_ok());
    }

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    crate::test_case! {support_flags_not_empty; supports_file_flags!(SF_APPEND, UF_APPEND)}
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    fn support_flags_not_empty(_: &mut TestContext) {}
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    #[test]
    fn support_flags_test_not_empty() {
        use std::collections::HashSet;

        use crate::flags::FileFlags;

        let mut config = Config::default();
        let tc: &TestCase = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::tests::t::support_flags_not_empty")
            .unwrap();
        assert_eq!(tc.guards.len(), 1);

        let guard = &tc.guards[0];
        assert!(guard(&config, Path::new("test")).is_err());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_err());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND, FileFlags::UF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_ok());

        config.features.file_flags = HashSet::from([
            FileFlags::SF_APPEND,
            FileFlags::UF_APPEND,
            FileFlags::SF_ARCHIVED,
        ]);
        assert!(guard(&config, Path::new("test")).is_ok());
    }
}
