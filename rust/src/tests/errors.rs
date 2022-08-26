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
mod eloop;
mod emlink;
mod enametoolong;
mod enoent;
mod enospc;
mod enotdir;
mod eperm;
mod erofs;
mod etxtbsy;

crate::test_case! {eisdir}
fn eisdir(ctx: &mut TestContext) {
    fn assert_eisdir<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let path = ctx.create(FileType::Dir).unwrap();
        assert_eq!(f(&path).unwrap_err(), Errno::EISDIR);
    }

    // open/13.t
    assert_eisdir(ctx, |p| open(p, OFlag::O_WRONLY, Mode::empty()));
    assert_eisdir(ctx, |p| open(p, OFlag::O_RDWR, Mode::empty()));
    assert_eisdir(ctx, |p| {
        open(p, OFlag::O_RDONLY | OFlag::O_TRUNC, Mode::empty())
    });
    assert_eisdir(ctx, |p| {
        open(p, OFlag::O_WRONLY | OFlag::O_TRUNC, Mode::empty())
    });
    assert_eisdir(ctx, |p| {
        open(p, OFlag::O_RDWR | OFlag::O_TRUNC, Mode::empty())
    });

    // (f)truncate/09.t
    assert_eisdir(ctx, |p| truncate(p, 0));
}

crate::test_case! {
    // rename/14.t
    eisdir_rename => [Regular, Fifo, Block, Char, Socket, Symlink(None)]
}
fn eisdir_rename(ctx: &mut TestContext, ft: FileType) {
    let dir = ctx.create(FileType::Dir).unwrap();
    let not_dir_file = ctx.create(ft).unwrap();
    assert_eq!(rename(&not_dir_file, &dir).unwrap_err(), Errno::EISDIR);
}

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

crate::test_case! {einval}
fn einval(ctx: &mut TestContext) {
    fn assert_einval<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let path = ctx.create(FileType::Regular).unwrap();
        assert_eq!(f(&path).unwrap_err(), Errno::EINVAL);
    }

    // (f)truncate/13.t
    assert_einval(ctx, |p| {
        let file = open(p, OFlag::O_RDWR, Mode::empty()).unwrap();
        ftruncate(file, -1)
    });
    assert_einval(ctx, |p| {
        let file = open(p, OFlag::O_WRONLY, Mode::empty()).unwrap();
        ftruncate(file, off_t::MIN)
    });

    // rename/19.t
    let dir = ctx.create(FileType::Dir).unwrap();
    let subdir = ctx.create_named(FileType::Dir, dir.join("subdir")).unwrap();
    assert!(matches!(
        rename(&subdir.join("."), &subdir.join("test")).unwrap_err(),
        Errno::EINVAL | Errno::EBUSY
    ));
    assert!(matches!(
        rename(&subdir.join(".."), &subdir.join("test")).unwrap_err(),
        Errno::EINVAL | Errno::EBUSY
    ));

    // rename/18.t
    let nested_subdir = ctx
        .create_named(FileType::Dir, subdir.join("nested"))
        .unwrap();
    assert_eq!(rename(&dir, &subdir).unwrap_err(), Errno::EINVAL);
    assert_eq!(rename(&dir, &nested_subdir).unwrap_err(), Errno::EINVAL);

    // (f)truncate/13.t
    assert_einval(ctx, |p| truncate(p, -1));
    assert_einval(ctx, |p| truncate(p, off_t::MIN));
}

crate::test_case! {
    /// open may return EINVAL when an attempt was made to open a descriptor
    /// with an illegal combination of O_RDONLY, O_WRONLY, and O_RDWR
    // open/23.t
    open_einval
}
fn open_einval(ctx: &mut TestContext) {
    fn assert_einval_open(ctx: &mut TestContext, flags: OFlag) {
        let path = ctx.create(FileType::Regular).unwrap();
        assert!(matches!(
            open(&path, flags, Mode::empty()),
            Ok(_) | Err(Errno::EINVAL)
        ));
    }

    assert_einval_open(ctx, OFlag::O_RDONLY | OFlag::O_RDWR);
    assert_einval_open(ctx, OFlag::O_WRONLY | OFlag::O_RDWR);
    assert_einval_open(ctx, OFlag::O_RDONLY | OFlag::O_WRONLY | OFlag::O_RDWR);
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
