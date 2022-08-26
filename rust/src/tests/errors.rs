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
    /// chmod returns EFTYPE if the effective user ID is not the super-user, the mode includes the sticky bit (S_ISVTX), and path does not refer to a directory
    // chmod/12.t
    eftype, serialized, root => [Regular, Fifo, Block, Char, Socket]
}
fn eftype(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();

    let original_mode = Mode::from_bits_truncate(0o640);
    let file = ctx
        .new_file(ft)
        .mode(original_mode.bits())
        .create()
        .unwrap();
    chown(&file, Some(user.uid), Some(user.gid)).unwrap();
    let new_mode = Mode::from_bits_truncate(0o644);
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    // TODO: Should be configured by the user? What to do with other OS?
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    {
        use crate::utils::lchmod;

        ctx.as_user(&user, None, || {
            assert_eq!(chmod(&file, new_mode | Mode::S_ISVTX), Err(Errno::EFTYPE));
        });
        let file_stat = stat(&file).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());

        ctx.as_user(&user, None, || {
            assert_eq!(
                chmod(&link, original_mode | Mode::S_ISVTX),
                Err(Errno::EFTYPE)
            );
        });
        let file_stat = stat(&link).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());

        // lchmod

        let mode = Mode::from_bits_truncate(0o621) | Mode::S_ISVTX;
        ctx.as_user(&user, None, || {
            assert_eq!(lchmod(&file, mode), Err(Errno::EFTYPE));
        });

        let file_stat = lstat(&file).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        ctx.as_user(&user, None, || {
            assert!(chmod(&file, new_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&file).unwrap();
        assert_eq!(
            file_stat.st_mode & ALLPERMS_STICKY,
            (new_mode | Mode::S_ISVTX).bits()
        );

        ctx.as_user(&user, None, || {
            assert!(chmod(&link, original_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&link).unwrap();
        assert_eq!(
            file_stat.st_mode & ALLPERMS_STICKY,
            (original_mode | Mode::S_ISVTX).bits()
        );
    }

    #[cfg(any(target_os = "solaris"))]
    {
        ctx.as_user(&user, None, || {
            assert!(chmod(&file, new_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&file).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, new_mode.bits());

        ctx.as_user(&user, None, || {
            assert!(chmod(&link, original_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&link).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());
    }
}

crate::test_case! {
    /// rmdir returns EINVAL if the last component of the path is '.'
    // rmdir/12.t
    rmdir_einval
}
fn rmdir_einval(ctx: &mut TestContext) {
    assert_eq!(rmdir(&ctx.base_path().join(".")), Err(Errno::EINVAL));
}
