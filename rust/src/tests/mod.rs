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
pub mod posix_fallocate;
pub mod unlink;
pub mod utimensat;

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

/// Assert that a certain operation changes the ctime of a file.
fn assert_ctime_changed<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    let ctime_before = metadata(&path).unwrap().ctime_ts();

    ctx.nap();

    f();

    let ctime_after = metadata(&path).unwrap().ctime_ts();
    assert!(ctime_after > ctime_before);
}

/// Assert that a certain operation changes the ctime of a file.
fn assert_mtime_changed<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    let mtime_before = metadata(&path).unwrap().mtime_ts();

    ctx.nap();

    f();

    let mtime_after = metadata(&path).unwrap().mtime_ts();
    assert!(mtime_after > mtime_before);
}

/// Assert that a certain operation does not change the ctime of a file.
fn assert_ctime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
where
    F: FnOnce(),
{
    let ctime_before = metadata(&path).unwrap().ctime_ts();

    ctx.nap();

    f();

    let ctime_after = metadata(&path).unwrap().ctime_ts();
    assert!(ctime_after == ctime_before);
}
