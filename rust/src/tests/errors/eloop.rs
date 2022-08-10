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

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::{
    runner::context::{FileType, TestContext},
    utils::{chmod, lchown, link, rename, rmdir, symlink},
};

/// Create a loop with two symlinks
fn create_loop_symlinks(ctx: &mut TestContext) -> (PathBuf, PathBuf) {
    let base_dir = ctx.create(FileType::Dir).unwrap();
    let loop1 = base_dir.join("loop1");
    let loop2 = base_dir.join("loop2");

    (
        ctx.create_named(FileType::Symlink(Some(loop2.clone())), &loop1)
            .unwrap(),
        ctx.create_named(FileType::Symlink(Some(loop1)), loop2)
            .unwrap(),
    )
}

crate::test_case! {
    /// Return ELOOP if too many symbolic links were encountered in translating a component of the pathname
    component
}
fn component(ctx: &mut TestContext) {
    /// Assert that it returns ELOOP if too many symbolic links were encountered in translating a component of the pathname
    fn assert_eloop_comp<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let (p1, p2) = create_loop_symlinks(ctx);

        assert_eq!(f(&p1.join("test")).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p2.join("test")).unwrap_err(), Errno::ELOOP);
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
        // chflags/06.t
        assert_eloop_comp(ctx, |p| chflags(p, FileFlag::empty()));
    }

    // chmod/06.t
    assert_eloop_comp(ctx, |p| chmod(p, Mode::empty()));

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        // chmod/06.t#L25
        assert_eloop_comp(ctx, |p| lchmod(p, Mode::empty()));
    }

    // chown/06.t
    assert_eloop_comp(ctx, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });

    // chown/06.t#L25
    assert_eloop_comp(ctx, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });

    // mkdir/07.t
    assert_eloop_comp(ctx, |p| mkdir(p, Mode::empty()));

    // mkfifo/07.t
    assert_eloop_comp(ctx, |p| mkfifo(p, Mode::empty()));

    // mknod/07.t
    assert_eloop_comp(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // open/12.t
    assert_eloop_comp(ctx, |p| open(p, OFlag::empty(), Mode::empty()));

    // rmdir/05.t
    assert_eloop_comp(ctx, rmdir);

    // symlink/07.t
    assert_eloop_comp(ctx, |p| symlink(Path::new("test"), p));

    // truncate/07.t
    assert_eloop_comp(ctx, |p| truncate(p, 0));

    // unlink/07.t
    assert_eloop_comp(ctx, unlink);
}

crate::test_case! {
    /// Returns ELOOP if too many symbolic links were encountered in translating the end of either pathname
    two_params
}
fn two_params(ctx: &mut TestContext) {
    /// Assert that it returns ELOOP if too many symbolic links were encountered in translating the end of either pathname
    fn assert_eloop_two_params<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let (p1, p2) = create_loop_symlinks(ctx);
        let valid_path = ctx.create(FileType::Regular).unwrap();

        assert_eq!(f(&p1.join("test"), &valid_path).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p2.join("test"), &valid_path).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&valid_path, &p1.join("test")).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&valid_path, &p2.join("test")).unwrap_err(), Errno::ELOOP);
    }

    // link/08.t
    assert_eloop_two_params(ctx, link);
    // reanme/11.t
    assert_eloop_two_params(ctx, rename);
}

crate::test_case! {
    /// Return ELOOP if too many symbolic links were encountered in translating the end of the pathname
    final_comp
}
fn final_comp(ctx: &mut TestContext) {
    /// Assert that it returns ELOOP if too many symbolic links were encountered in translating the end of the pathname
    fn assert_eloop_final<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let (p1, p2) = create_loop_symlinks(ctx);

        assert_eq!(f(&p1).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p2).unwrap_err(), Errno::ELOOP);
    }

    // chmod/06.t
    assert_eloop_final(ctx, |p| chmod(p, Mode::empty()));
    // chown/06.t
    assert_eloop_final(ctx, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
}
//TODO: open/16.t
