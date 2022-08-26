use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, Mode, SFlag},
    unistd::{chown, mkdir, mkfifo, truncate, unlink, User},
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
    runner::context::{FileType, SerializedTestContext},
    utils::{chmod, lchown, link, rename, rmdir, symlink},
};

mod open;
mod sticky;

crate::test_case! {
    /// Return EACCES when search permission is denied for a component of the path prefix
    search_perm, serialized, root
}
fn search_perm(ctx: &mut SerializedTestContext) {
    /// Asserts that it returns EACCES when search permission is denied for a component of the path prefix
    fn assert_eacces_search_perm<F, T: Debug>(ctx: &mut SerializedTestContext, user: &User, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.new_file(FileType::Dir).mode(0o644).create().unwrap();
        let path = dir.join("test");

        ctx.as_user(user, None, || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    let user = ctx.get_new_user();

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    {
        // chflags/05.t
        assert_eacces_search_perm(ctx, &user, |p| chflags(p, FileFlag::empty()));
    }

    // chmod/05.t
    assert_eacces_search_perm(ctx, &user, |p| chmod(p, Mode::empty()));
    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        assert_eacces_search_perm(ctx, &user, |p| lchmod(p, Mode::empty()));
    }

    // chown/05.t
    assert_eacces_search_perm(ctx, &user, |p| chown(p, Some(user.uid), None));
    assert_eacces_search_perm(ctx, &user, |p| lchown(p, Some(user.uid), None));

    // mkdir/05.t
    assert_eacces_search_perm(ctx, &user, |p| mkdir(p, Mode::empty()));

    // mkfifo/05.t
    assert_eacces_search_perm(ctx, &user, |p| mkfifo(p, Mode::empty()));

    // mknod/05.t
    assert_eacces_search_perm(ctx, &user, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // open/05.t
    assert_eacces_search_perm(ctx, &user, |p| open(p, OFlag::O_RDONLY, Mode::empty()));

    // rmdir/07.t
    assert_eacces_search_perm(ctx, &user, rmdir);

    // symlink/05.t
    assert_eacces_search_perm(ctx, &user, |p| symlink(Path::new("test"), p));

    // (f)truncate/06.t
    assert_eacces_search_perm(ctx, &user, |p| truncate(p, 0));

    // unlink/05.t
    assert_eacces_search_perm(ctx, &user, unlink);
}

crate::test_case! {
    /// Return EACCES if the named file is not writable by the user
    named_file, serialized, root
}
fn named_file(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();

    /// Asserts that it returns EACCES if the named file is not writable by the user
    fn assert_eacces_write_perm_file<F, T: Debug>(
        ctx: &mut SerializedTestContext,
        user: &User,
        f: F,
    ) where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.new_file(FileType::Dir).mode(0o444).create().unwrap();
        let path = dir.join("test");

        ctx.as_user(user, None, || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    // (f)truncate/05.t
    assert_eacces_write_perm_file(ctx, &user, |p| truncate(p, 0));

    // unlink/06.t
    assert_eacces_write_perm_file(ctx, &user, unlink);

    // rmdir/08.t
    assert_eacces_write_perm_file(ctx, &user, rmdir);
}

crate::test_case! {
    /// Return EACCES when write permission is denied on the parent directory of the directory to be created
    write_perm_parent, serialized, root
}
fn write_perm_parent(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();

    /// Asserts that it returns EACCES when write permission is denied on the parent directory of the directory to be created
    fn assert_eacces_write_perm<F, T: Debug>(ctx: &mut SerializedTestContext, user: &User, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.new_file(FileType::Dir).mode(0o555).create().unwrap();
        let path = dir.join("test");

        ctx.as_user(user, None, || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    // mkdir/06.t
    assert_eacces_write_perm(ctx, &user, |p| mkdir(p, Mode::empty()));

    // mkfifo/06.t
    assert_eacces_write_perm(ctx, &user, |p| mkfifo(p, Mode::empty()));

    // mknod/06.t
    assert_eacces_write_perm(ctx, &user, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // open/08.t
    assert_eacces_write_perm(ctx, &user, |p| {
        open(p, OFlag::O_CREAT | OFlag::O_RDONLY, Mode::empty())
    });

    // symlink/06.t
    assert_eacces_write_perm(ctx, &user, |p| symlink(Path::new("test"), p));
}

crate::test_case! {
    /// Return EACCES when the requested link requires writing in a directory with a mode that denies write permission
    write_dir_write_perm, serialized, root
}
fn write_dir_write_perm(ctx: &mut SerializedTestContext) {
    fn assert_write_perm<F, T: Debug>(ctx: &mut SerializedTestContext, user: &User, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let from_dir = ctx.create(FileType::Dir).unwrap();
        chown(&from_dir, Some(user.uid), Some(user.gid)).unwrap();

        let from_path = ctx
            .create_named(FileType::Regular, from_dir.join("file"))
            .unwrap();
        chown(&from_path, Some(user.uid), Some(user.gid)).unwrap();
        let to_same_dir = from_dir.join("file1");

        let to_dir = ctx.create(FileType::Dir).unwrap();
        chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();

        let to_path = to_dir.join("file");

        //TODO: Test that it succeed first? it's already done in the other tests?

        chmod(&to_dir, Mode::from_bits_truncate(0o555)).unwrap();
        ctx.as_user(user, None, || {
            assert_eq!(f(&from_path, &to_path).unwrap_err(), Errno::EACCES);
        });

        chmod(&to_dir, Mode::from_bits_truncate(0o755)).unwrap();
        chmod(&from_dir, Mode::from_bits_truncate(0o555)).unwrap();
        ctx.as_user(user, None, || {
            assert_eq!(f(&from_path, &to_same_dir).unwrap_err(), Errno::EACCES);
        });
    }

    let user = ctx.get_new_user();

    assert_write_perm(ctx, &user, link);
    assert_write_perm(ctx, &user, rename);
}
crate::test_case! {
    /// Return EACCES when search permission is denied for a component of either path prefix
    search_perm_either, serialized, root
}
fn search_perm_either(ctx: &mut SerializedTestContext) {
    fn assert_write_perm<F, T: Debug>(ctx: &mut SerializedTestContext, user: &User, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let from_dir = ctx.create(FileType::Dir).unwrap();
        chown(&from_dir, Some(user.uid), Some(user.gid)).unwrap();

        let from_path = ctx
            .create_named(FileType::Regular, from_dir.join("file"))
            .unwrap();
        chown(&from_path, Some(user.uid), Some(user.gid)).unwrap();
        let to_same_dir = from_dir.join("file1");

        let to_dir = ctx.create(FileType::Dir).unwrap();
        chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();

        let to_path = to_dir.join("file");

        //TODO: Test that it succeed first? it's already done in the other tests?

        chmod(&to_dir, Mode::from_bits_truncate(0o644)).unwrap();
        ctx.as_user(user, None, || {
            assert_eq!(f(&from_path, &to_path).unwrap_err(), Errno::EACCES);
        });

        chmod(&to_dir, Mode::from_bits_truncate(0o755)).unwrap();
        chmod(&from_dir, Mode::from_bits_truncate(0o644)).unwrap();
        ctx.as_user(user, None, || {
            assert_eq!(f(&from_path, &to_same_dir).unwrap_err(), Errno::EACCES);
        });
    }

    let user = ctx.get_new_user();

    assert_write_perm(ctx, &user, link);
    assert_write_perm(ctx, &user, rename);
}
