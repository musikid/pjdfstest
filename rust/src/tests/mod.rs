use std::fs::symlink_metadata;
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

pub mod chmod;
pub mod ftruncate;
pub mod link;
pub mod mkdir;
pub mod mkfifo;
pub mod mknod;
mod mksyscalls;
pub mod posix_fallocate;
pub mod rename;
pub mod rmdir;
pub mod symlink;
pub mod unlink;
pub mod utimensat;

#[derive(Debug, PartialEq, PartialOrd)]
struct TimeMetadata {
    atime_ts: Option<TimeSpec>,
    ctime_ts: Option<TimeSpec>,
    mtime_ts: Option<TimeSpec>,
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

    fn time_meta(&self, atime: bool, ctime: bool, mtime: bool) -> TimeMetadata {
        TimeMetadata {
            atime_ts: atime.then(|| self.atime_ts()),
            ctime_ts: ctime.then(|| self.ctime_ts()),
            mtime_ts: mtime.then(|| self.mtime_ts()),
        }
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
/// which compares metadata of one file `before` to many files `after`.
struct TimeAssertion<'a, F> {
    before_path: &'a Path,
    after_paths: Vec<&'a Path>,
    atime: bool,
    ctime: bool,
    mtime: bool,
    equal: bool,
    no_follow_symlink: bool,
    fun: F,
}

impl<'a, F> TimeAssertion<'a, F>
where
    F: FnOnce(),
{
    /// Return a new builder with the provided path being the `before` compared path
    /// and added to the `after` compared paths.
    /// Comparision will be an equality check if `equal` is true, or an ordering one if it is false.
    pub fn new<P: AsRef<Path>>(path: &'a P, equal: bool, fun: F) -> Self {
        Self {
            before_path: path.as_ref(),
            after_paths: vec![path.as_ref()],
            atime: false,
            ctime: false,
            mtime: false,
            no_follow_symlink: false,
            equal,
            fun,
        }
    }

    /// Return a new builder with the provided path being the `before` compared path.
    /// Comparision will be an equality check if `equal` is true, or an ordering one if it is false.
    pub fn new_with_paths<P: AsRef<Path>>(before_path: &'a P, equal: bool, fun: F) -> Self {
        Self {
            before_path: before_path.as_ref(),
            after_paths: vec![],
            atime: false,
            ctime: false,
            mtime: false,
            no_follow_symlink: false,
            equal,
            fun,
        }
    }

    /// Add a path to the `after` compared paths.
    pub fn add_after<P: AsRef<Path>>(mut self, path: &'a P) -> Self {
        self.after_paths.push(path.as_ref());
        self
    }

    /// Add the `st_atime` field to those being compared.
    pub fn atime(mut self) -> Self {
        self.atime = true;
        self
    }

    /// Add the `st_ctime` field to those being compared.
    pub fn ctime(mut self) -> Self {
        self.ctime = true;
        self
    }

    /// Add the `st_mtime` field to those being compared.
    pub fn mtime(mut self) -> Self {
        self.mtime = true;
        self
    }

    /// Get metadata without following symlinks.
    pub fn no_follow_symlink(mut self) -> Self {
        self.no_follow_symlink = true;
        self
    }

    /// Build the assertion and asserts that `before` metadata
    /// is either equal or before the `after` metadata.
    pub fn assert(self, ctx: &TestContext) {
        if !(self.atime || self.ctime || self.mtime) || self.after_paths.is_empty() {
            unimplemented!()
        }

        let get_metadata = if self.no_follow_symlink {
            symlink_metadata
        } else {
            metadata
        };

        let meta_before = get_metadata(self.before_path)
            .unwrap()
            .time_meta(self.atime, self.ctime, self.mtime);

        ctx.nap();

        (self.fun)();

        let metas_after: Vec<_> = self
            .after_paths
            .into_iter()
            .map(|p| {
                get_metadata(p)
                    .unwrap()
                    .time_meta(self.atime, self.ctime, self.mtime)
            })
            .collect();

        for meta_after in metas_after {
            if self.equal {
                assert_eq!(meta_before, meta_after);
            } else {
                assert!(meta_after > meta_before);
            }
        }
    }
}

/// Assert that a certain operation changes the ctime of a file.
fn assert_ctime_changed<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    TimeAssertion::new(&path, false, f).ctime().assert(ctx)
}

/// Assert that a certain operation changes the mtime of a file.
fn assert_mtime_changed<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    TimeAssertion::new(&path, false, f).mtime().assert(ctx)
}

/// Assert that a certain operation does not change the ctime of a file.
fn assert_ctime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    TimeAssertion::new(&path, true, f).ctime().assert(ctx)
}

/// Assert that a certain operation does not change the ctime of a file without following symlinks.
fn assert_symlink_ctime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    TimeAssertion::new(&path, true, f)
        .ctime()
        .no_follow_symlink()
        .assert(ctx)
}

/// Assert that a certain operation does not change the mtime of a file.
fn assert_mtime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    TimeAssertion::new(&path, true, f).mtime().assert(ctx)
}
