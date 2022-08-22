use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, Mode, SFlag},
    unistd::{mkdir, mkfifo},
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

use std::{fmt::Debug, path::Path};

use crate::{
    runner::context::{FileType, TestContext},
    utils::{link, rename, rmdir, symlink},
};

/// Asserts that it returns EEXIST if the named file exists
fn assert_eexist<F, T: Debug>(ctx: &mut TestContext, ft: &FileType, f: F)
where
    F: Fn(&Path) -> nix::Result<T>,
{
    let path = ctx.create(ft.clone()).unwrap();
    assert_eq!(f(&path).unwrap_err(), Errno::EEXIST);
}

crate::test_case! {
    /// Return EEXIST if the named file exists
    named_file => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn named_file(ctx: &mut TestContext, ft: FileType) {
    //TODO: Add rmdir/{06,12}.t

    let default_mode = Mode::from_bits_truncate(0o644);

    // mkdir/10.t
    assert_eexist(ctx, &ft, |p| mkdir(p, Mode::empty()));

    // mknod/08.t
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // mkfifo/09.t
    assert_eexist(ctx, &ft, |p| mkfifo(p, default_mode));

    // link/10.t
    let regular_file = ctx.create(FileType::Regular).unwrap();
    assert_eexist(ctx, &ft, |p| link(&*regular_file, p));

    // open/22.t
    assert_eexist(ctx, &ft, |p| {
        open(p, OFlag::O_CREAT | OFlag::O_EXCL, default_mode)
    });

    // symlink/08.t
    assert_eexist(ctx, &ft, |p| symlink(Path::new("test"), p));
}

crate::test_case! {
    /// rmdir returns EEXIST or ENOTEMPTY if the named directory contains files other than '.' and '..' in it
    /// or if the last component of the path is '..'
    rmdir_enotempty => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn rmdir_enotempty(ctx: &mut TestContext, ft: FileType) {
    // rmdir/06.t
    ctx.create(ft).unwrap();
    assert!(matches!(
        rmdir(ctx.base_path()),
        Err(Errno::ENOTEMPTY | Errno::EEXIST)
    ));
    // rmdir/12.t
    // TODO: Not conforming to POSIX on FreeBSD
    #[cfg(not(target_os = "freebsd"))]
    {
        let dir = ctx.create(FileType::Dir).unwrap();
        assert!(matches!(
            rmdir(&dir.join("..")),
            Err(Errno::ENOTEMPTY | Errno::EEXIST)
        ));
    }
}

crate::test_case! {
    /// rename returns EEXIST or ENOTEMPTY if the 'to' argument is a directory and is not empty
    from_to_rename => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn from_to_rename(ctx: &mut TestContext, ft: FileType) {
    let from_dir = ctx.create(FileType::Dir).unwrap();
    let to_dir = ctx.create(FileType::Dir).unwrap();
    ctx.create_named(ft, to_dir.join("test")).unwrap();
    assert!(matches!(
        rename(&from_dir, &to_dir).unwrap_err(),
        Errno::EEXIST | Errno::ENOTEMPTY
    ));
}

crate::test_case! {privileged, root => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]}
fn privileged(ctx: &mut TestContext, ft: FileType) {
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
}
