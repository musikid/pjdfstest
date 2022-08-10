use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, Mode, SFlag},
    unistd::{chown, mkdir, mkfifo, truncate, unlink},
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

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::{
    runner::context::{FileType, TestContext},
    utils::{chmod, lchown, link, rename, rmdir, symlink},
};

fn create_fake_path(ctx: &mut TestContext) -> PathBuf {
    let dir = ctx.create(FileType::Dir).unwrap();
    dir.join("not_existent")
}

crate::test_case! {
    /// Return ENOENT if the named file does not exist
    named_file
}
fn named_file(ctx: &mut TestContext) {
    /// Asserts that it returns ENOENT if the named file does not exist
    fn assert_enoent<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let fake_path = create_fake_path(ctx);
        assert_eq!(f(&fake_path).unwrap_err(), Errno::ENOENT);
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
        // chflags/04.t
        assert_enoent(ctx, |p| chflags(p, FileFlag::empty()));
    }

    // chmod/04.t
    assert_enoent(ctx, |p| chmod(p, Mode::empty()));

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        // chmod/04.t
        assert_enoent(ctx, |p| lchmod(p, Mode::empty()));
    }

    let user = ctx.get_new_user();

    // chown/04.t
    assert_enoent(ctx, |p| chown(p, Some(user.uid), None));
    assert_enoent(ctx, |p| lchown(p, Some(user.uid), None));

    // open/04.t
    assert_enoent(ctx, |p| open(p, OFlag::O_RDONLY, Mode::empty()));

    // rmdir/04.t
    assert_enoent(ctx, rmdir);

    // (f)truncate/04.t
    assert_enoent(ctx, |p| truncate(p, 0));

    // unlink/04.t
    assert_enoent(ctx, unlink);
}

crate::test_case! {
    /// Return ENOENT if the symlink target named file does not exist
    symlink_named_file
}
fn symlink_named_file(ctx: &mut TestContext) {
    /// Asserts that it returns ENOENT if the symlink target named file does not exist
    fn assert_enoent_link<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let fake_path = create_fake_path(ctx);
        let link_to_fake_path = ctx
            .create(FileType::Symlink(Some(fake_path.to_path_buf())))
            .unwrap();
        assert_eq!(f(&link_to_fake_path).unwrap_err(), Errno::ENOENT);
    }

    // chmod/04.t
    assert_enoent_link(ctx, |p| chmod(p, Mode::empty()));
    // chown/04.t
    let user = ctx.get_new_user();
    assert_enoent_link(ctx, |p| chown(p, Some(user.uid), None));
}

crate::test_case! {
    /// Return ENOENT if a component of either path prefix does not exist
    two_params
}
fn two_params(ctx: &mut TestContext) {
    /// Asserts that it returns ENOENT if a component of either path prefix does not exist
    fn assert_enoent_two_params<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let fake_path = create_fake_path(ctx);
        let real_path = ctx.create(FileType::Regular).unwrap();

        assert_eq!(
            f(&fake_path.join("test"), &real_path).unwrap_err(),
            Errno::ENOENT
        );
        assert_eq!(
            f(&real_path, &fake_path.join("test")).unwrap_err(),
            Errno::ENOENT
        );
    }

    // link/04.t
    assert_enoent_two_params(ctx, link);

    // rename/03.t
    assert_enoent_two_params(ctx, rename);
}

crate::test_case! {
    /// Return ENOENT if a component of the path prefix does not exist
    final_comp
}
fn final_comp(ctx: &mut TestContext) {
    /// Asserts that it returns ENOENT if a component of the path prefix does not exist
    fn assert_enoent_final_comp<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let fake_path = create_fake_path(ctx);
        assert_eq!(f(&fake_path.join("test")).unwrap_err(), Errno::ENOENT);
    }

    // chmod/04.t
    assert_enoent_final_comp(ctx, |p| chmod(p, Mode::empty()));

    // chown/04.t
    let user = ctx.get_new_user();
    assert_enoent_final_comp(ctx, |p| chown(p, Some(user.uid), None));

    // mkdir/04.t
    assert_enoent_final_comp(ctx, |p| mkdir(p, Mode::empty()));

    // mknod/04.t
    assert_enoent_final_comp(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // mkfifo/04.t
    assert_enoent_final_comp(ctx, |p| mkfifo(p, Mode::empty()));

    // open/04.t
    assert_enoent_final_comp(ctx, |p| open(p, OFlag::O_CREAT, Mode::S_IRWXU));

    // symlink/04.t
    assert_enoent_final_comp(ctx, |p| symlink(Path::new("test"), p));
}

//TODO: DO link/09.t?
