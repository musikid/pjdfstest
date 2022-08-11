use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, Mode, SFlag},
    unistd::{chown, mkdir, mkfifo, truncate, unlink},
};

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
use crate::utils::lchmod;
#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios"
))]
use nix::{sys::stat::FileFlag, unistd::chflags};

use std::{fmt::Debug, path::Path};

use crate::{
    runner::context::{FileType, TestContext},
    utils::{chmod, lchown, link, rename, rmdir, symlink},
};

crate::test_case! {
    /// Return ENOTDIR if a component of either path prefix is not a directory
    two_params => [Regular, Fifo, Block, Char, Socket]
}
fn two_params(ctx: &mut TestContext, ft: FileType) {
    /// Asserts that it returns ENOTDIR if a component of either path prefix is not a directory.
    fn assert_enotdir_two_params<T: Debug, F>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let base_path = ctx.create(ft.clone()).unwrap();
        let path = base_path.join("previous_not_dir");
        let new_path = ctx.base_path().join("not_existent");

        assert_eq!(f(&path, &new_path).unwrap_err(), Errno::ENOTDIR);

        let file = ctx.create(ft.clone()).unwrap();

        assert_eq!(f(&file, &path).unwrap_err(), Errno::ENOTDIR);
    }

    // link/01.t
    assert_enotdir_two_params(ctx, &ft, link);

    // rename/12.t
    assert_enotdir_two_params(ctx, &ft, rename);
}

crate::test_case! {
    /// Return ENOTDIR when the 'from' argument is a directory, but 'to' is not a directory
    from_to => [Regular, Fifo, Block, Char, Socket]
}
fn from_to(ctx: &mut TestContext, ft: FileType) {
    /// Asserts that it returns ENOTDIR when the 'from' argument is a directory, but 'to' is not a directory.
    fn assert_enotdir_from_to<T: Debug, F>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let path = ctx.create(ft.clone()).unwrap();
        let dir = ctx.create(FileType::Dir).unwrap();

        assert_eq!(f(&dir, &path).unwrap_err(), Errno::ENOTDIR);
    }

    // rename/13.t
    assert_enotdir_from_to(ctx, &ft, rename);
}

crate::test_case! {
    /// Return ENOTDIR if a component of the path prefix is not a directory
    component => [Regular, Fifo, Block, Char, Socket]
}
fn component(ctx: &mut TestContext, ft: FileType) {
    /// Asserts that it returns ENOTDIR if a component of the path prefix is not a directory.
    fn assert_enotdir<T: Debug, F>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let base_path = ctx.create(ft.clone()).unwrap();
        let path = base_path.join("previous_not_dir");

        assert_eq!(f(&path).unwrap_err(), Errno::ENOTDIR);
    }

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    {
        // chflags/01.t
        assert_enotdir(ctx, &ft, |p| chflags(p, FileFlag::empty()));
    }

    // chmod/01.t
    assert_enotdir(ctx, &ft, |p| chmod(p, Mode::empty()));
    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        assert_enotdir(ctx, &ft, |p| lchmod(p, Mode::empty()));
    }
    // chown/01.t
    let user = ctx.get_new_user();
    assert_enotdir(ctx, &ft, |p| chown(p, Some(user.uid), None));
    assert_enotdir(ctx, &ft, |p| lchown(p, Some(user.uid), None));
    // mkdir/01.t
    assert_enotdir(ctx, &ft, |p| mkdir(p, Mode::empty()));
    // mkfifo/01.t
    assert_enotdir(ctx, &ft, |p| mkfifo(p, Mode::empty()));
    // mknod/01.t
    assert_enotdir(ctx, &ft, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    // open/01.t
    assert_enotdir(ctx, &ft, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    assert_enotdir(ctx, &ft, |p| {
        open(p, OFlag::O_CREAT, Mode::from_bits_truncate(0o644))
    });
    // rmdir/01.t
    assert_enotdir(ctx, &ft, rmdir);
    // symlink/01.t
    assert_enotdir(ctx, &ft, |p| symlink(Path::new("test"), p));
    // (f)truncate/01.t
    assert_enotdir(ctx, &ft, |p| truncate(p, 0));
    // unlink/01.t
    assert_enotdir(ctx, &ft, unlink);
}

crate::test_case! {
    /// Return ENOTDIR if a component of the path prefix is not a directory
    mknod_privileged, root => [Regular, Fifo, Block, Char, Socket]
}
fn mknod_privileged(ctx: &mut TestContext, ft: FileType) {
    let base_path = ctx.create(ft.clone()).unwrap();
    let path = base_path.join("previous_not_dir");

    // mknod/01.t
    assert_eq!(
        mknod(&path, SFlag::S_IFCHR, Mode::empty(), 0).unwrap_err(),
        Errno::ENOTDIR
    );
    assert_eq!(
        mknod(&path, SFlag::S_IFBLK, Mode::empty(), 0).unwrap_err(),
        Errno::ENOTDIR
    );
}
