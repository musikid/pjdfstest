use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, Mode, SFlag},
    unistd::{mkdir, mkfifo, truncate, unlink, User},
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
    runner::context::{FileType, SerializedTestContext},
    utils::{chmod, rmdir, symlink},
};

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
        let dir = ctx.create(FileType::Dir).unwrap();
        let mode = Mode::from_bits_truncate(0o644);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();

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

    // mkdir/05.t
    assert_eacces_search_perm(ctx, &user, |p| mkdir(p, Mode::empty()));

    // mkfifo/05.t
    assert_eacces_search_perm(ctx, &user, |p| mkfifo(p, Mode::empty()));

    // mknod/05.t
    assert_eacces_search_perm(ctx, &user, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    //TODO: open
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
        let dir = ctx.create(FileType::Dir).unwrap();
        let mode = Mode::from_bits_truncate(0o444);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();

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
        let dir = ctx.create(FileType::Dir).unwrap();
        let mode = Mode::from_bits_truncate(0o555);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();

        ctx.as_user(user, None, || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    // chown/05.t
    // TODO: Blocked by #63
    // assert_eacces(ctx,&user, |p| {
    //     chown(
    //         p,
    //         Some(User::from_name("nobody").unwrap().unwrap().uid),
    //         None,
    //     )
    // });
    // assert_eacces(ctx,&user, |p| {
    //     lchown(
    //         p,
    //         Some(User::from_name("nobody").unwrap().unwrap().uid),
    //         None,
    //     )
    // });
    // TODO: link specialization

    // mkdir/06.t
    assert_eacces_write_perm(ctx, &user, |p| mkdir(p, Mode::empty()));

    // mkfifo/06.t
    assert_eacces_write_perm(ctx, &user, |p| mkfifo(p, Mode::empty()));

    // mknod/06.t
    assert_eacces_write_perm(ctx, &user, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // TODO: open
    assert_eacces_write_perm(ctx, &user, |p| {
        open(p, OFlag::O_CREAT | OFlag::O_RDONLY, Mode::empty())
    });

    //TODO: rename

    // symlink/06.t
    assert_eacces_write_perm(ctx, &user, |p| symlink(Path::new("test"), p));
}
