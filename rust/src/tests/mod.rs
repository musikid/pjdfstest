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
pub mod posix_fallocate;
pub mod utimensat;

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)`.
pub fn chmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    nix::sys::stat::fchmodat(
        None,
        path,
        mode,
        nix::sys::stat::FchmodatFlags::FollowSymlink,
    )
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
fn assert_ctime_changed<const S: bool, F>(ctx: &mut TestContext<S>, path: &Path, f: F)
where
    F: FnOnce(),
{
    let ctime_before = metadata(&path).unwrap().ctime_ts();

    ctx.nap();

    f();

    let ctime_after = metadata(&path).unwrap().ctime_ts();
    assert!(ctime_after > ctime_before);
}

/// Assert that a certain operation does not change the ctime of a file.
fn assert_ctime_unchanged<const S: bool, F>(ctx: &TestContext<S>, path: &Path, f: F)
where
    F: FnOnce(),
{
    let ctime_before = metadata(&path).unwrap().ctime_ts();

    ctx.nap();

    f();

    let ctime_after = metadata(&path).unwrap().ctime_ts();
    assert!(ctime_after == ctime_before);
}
