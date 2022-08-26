use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    libc::off_t,
    sys::stat::{lstat, stat, Mode},
    unistd::{chown, ftruncate, truncate},
};

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios"
))]
use nix::{sys::stat::FileFlag, unistd::chflags};

use std::{fmt::Debug, fs::metadata, os::unix::prelude::PermissionsExt, path::Path};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    utils::{chmod, rename, rmdir, symlink, ALLPERMS, ALLPERMS_STICKY},
};

mod eacces;
mod eexist;
mod efault;
mod eftype;
mod einval;
mod eisdir;
mod eloop;
mod emlink;
mod enametoolong;
mod enoent;
mod enospc;
mod enotdir;
mod eperm;
mod erofs;
mod etxtbsy;

crate::test_case! {
    /// truncate must not change the file size if it fails with EFBIG or EINVAL
    /// because the length argument was greater than the maximum file size
    // (f)truncate/12.t
    truncate_efbig
}
fn truncate_efbig(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let size = 999999999999999;
    let res = truncate(&file, size);

    let expected_size = match res {
        Ok(_) => size,
        Err(Errno::EFBIG | Errno::EINVAL) => 0,
        Err(e) => panic!("truncate failed with {e}"),
    };

    let stat = stat(&file).unwrap();
    assert_eq!(stat.st_size, expected_size);
}

crate::test_case! {
    /// rmdir returns EINVAL if the last component of the path is '.'
    // rmdir/12.t
    rmdir_einval
}
fn rmdir_einval(ctx: &mut TestContext) {
    assert_eq!(rmdir(&ctx.base_path().join(".")), Err(Errno::EINVAL));
}
