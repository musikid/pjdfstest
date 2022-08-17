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

use std::{fmt::Debug, path::Path};

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
use crate::utils::lchmod;
use crate::{
    runner::context::{FileType, TestContext},
    utils::{chmod, lchown, link, rename, rmdir, symlink},
};

/// Asserts that it returns ENAMETOOLONG if a component of a pathname exceeded {NAME_MAX} characters
fn assert_enametoolong_comp<F, T: Debug + std::cmp::PartialEq>(ctx: &mut TestContext, f: F)
where
    F: Fn(&Path) -> nix::Result<T>,
{
    let mut invalid_path = ctx.create_name_max(FileType::Regular).unwrap();
    invalid_path.set_extension("x");
    assert_eq!(f(&invalid_path).unwrap_err(), Errno::ENAMETOOLONG);
}

/// Asserts that it returns ENAMETOOLONG if an entire path name exceeded {PATH_MAX} characters
fn assert_enametoolong_path<F, T: Debug + std::cmp::PartialEq>(ctx: &mut TestContext, f: F)
where
    F: Fn(&Path) -> nix::Result<T>,
{
    let mut invalid_path = ctx.create_path_max(FileType::Regular).unwrap();
    invalid_path.set_extension("x");
    assert_eq!(f(&invalid_path).unwrap_err(), Errno::ENAMETOOLONG);
}

crate::test_case! {
    /// Return ENAMETOOLONG if a component of a pathname exceeded {NAME_MAX} characters
    component
}
fn component(ctx: &mut TestContext) {
    //TODO: lchflags too?
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    // chflags/02.t
    assert_enametoolong_comp(ctx, |p| chflags(p, FileFlag::empty()));

    // chmod/02.t
    assert_enametoolong_comp(ctx, |p| chmod(p, Mode::empty()));
    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    assert_enametoolong_comp(ctx, |p| lchmod(p, Mode::empty()));

    // chown/02.t
    let user = ctx.get_new_user();
    assert_enametoolong_comp(ctx, |p| chown(p, Some(user.uid), Some(user.gid)));
    assert_enametoolong_comp(ctx, |p| lchown(p, Some(user.uid), Some(user.gid)));

    // mkfifo/02.t
    assert_enametoolong_comp(ctx, |p| mkfifo(p, Mode::empty()));

    // mkdir/02.t
    assert_enametoolong_comp(ctx, |p| mkdir(p, Mode::empty()));

    // mknod/02.t
    assert_enametoolong_comp(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // open/02.t
    assert_enametoolong_comp(ctx, |p| open(p, OFlag::O_CREAT, Mode::empty()));

    // rmdir/02.t
    assert_enametoolong_comp(ctx, rmdir);

    // symlink/02.t
    let valid_path = ctx.create(FileType::Regular).unwrap();
    assert_enametoolong_comp(ctx, |p| symlink(&*valid_path, p));

    // (f)truncate/02.t
    assert_enametoolong_comp(ctx, |p| truncate(p, 0));

    // unlink/02.t
    assert_enametoolong_comp(ctx, |p| unlink(p));
}

crate::test_case! {
    /// Return ENAMETOOLONG if a component of either pathname exceeded {NAME_MAX} characters
    component_two_params
}
fn component_two_params(ctx: &mut TestContext) {
    /// Asserts that it returns ENAMETOOLONG if a component of either pathname exceeded {NAME_MAX} characters
    fn assert_enametoolong_comp_two_params<F, T: Debug + std::cmp::PartialEq>(
        ctx: &mut TestContext,
        f: F,
    ) where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let mut invalid_path = ctx.create_name_max(FileType::Regular).unwrap();
        invalid_path.set_extension("x");
        let valid_path = ctx.create_path_max(FileType::Regular).unwrap();
        assert_eq!(
            f(&invalid_path, &valid_path).unwrap_err(),
            Errno::ENAMETOOLONG
        );
        assert_eq!(
            f(&invalid_path, &valid_path).unwrap_err(),
            Errno::ENAMETOOLONG
        );
    }

    // link/02.t
    assert_enametoolong_comp_two_params(ctx, link);

    // rename/01.t
    assert_enametoolong_comp_two_params(ctx, rename);
}

crate::test_case! {
    /// Return ENAMETOOLONG if an entire path name exceeded {PATH_MAX} characters
    pathname
}
fn pathname(ctx: &mut TestContext) {
    //TODO: lchflags too?
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    assert_enametoolong_path(ctx, |p| chflags(p, FileFlag::empty()));

    // chmod/03.t
    assert_enametoolong_path(ctx, |p| chmod(p, Mode::empty()));
    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    assert_enametoolong_path(ctx, |p| lchmod(p, Mode::empty()));

    // chown/03.t
    let user = ctx.get_new_user();
    assert_enametoolong_path(ctx, |p| chown(p, Some(user.uid), Some(user.gid)));
    assert_enametoolong_path(ctx, |p| lchown(p, Some(user.uid), Some(user.gid)));

    // mkfifo/03.t
    assert_enametoolong_path(ctx, |p| mkfifo(p, Mode::empty()));

    // mkdir/03.t
    assert_enametoolong_path(ctx, |p| mkdir(p, Mode::empty()));

    // mknod/03.t
    assert_enametoolong_path(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // open/03.t
    assert_enametoolong_path(ctx, |p| open(p, OFlag::O_CREAT, Mode::empty()));

    // rmdir/03.t
    assert_enametoolong_path(ctx, rmdir);

    // (f)truncate/03.t
    assert_enametoolong_path(ctx, |p| truncate(p, 0));

    // unlink/03.t
    assert_enametoolong_path(ctx, unlink);
}

crate::test_case! {privileged, root}
fn privileged(ctx: &mut TestContext) {
    // mknod/02.t
    assert_enametoolong_comp(ctx, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
    assert_enametoolong_comp(ctx, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
    // mknod/03.t
    assert_enametoolong_path(ctx, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
}

crate::test_case! {
    /// Return ENAMETOOLONG if the entire of either path name exceeded {PATH_MAX} characters
    pathname_two_params
}
fn pathname_two_params(ctx: &mut TestContext) {
    /// Asserts that it returns ENAMETOOLONG if the entire of either path name exceeded {PATH_MAX} characters
    fn assert_enametoolong_path_two_params<F, T: Debug + std::cmp::PartialEq>(
        ctx: &mut TestContext,
        f: F,
    ) where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let mut invalid_path = ctx.create_path_max(FileType::Regular).unwrap();
        invalid_path.set_extension("x");
        let valid_path = ctx.create_path_max(FileType::Regular).unwrap();
        assert_eq!(f(&invalid_path, &valid_path), Err(Errno::ENAMETOOLONG));
        assert_eq!(f(&invalid_path, &valid_path), Err(Errno::ENAMETOOLONG));
    }

    // link/03.t
    assert_enametoolong_path_two_params(ctx, link);

    // rename/02.t
    assert_enametoolong_path_two_params(ctx, rename);
}
