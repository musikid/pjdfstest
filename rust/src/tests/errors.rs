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
mod efbig;
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
