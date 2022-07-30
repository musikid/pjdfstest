use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    libc::off_t,
    sys::stat::{mknod, Mode, SFlag},
    unistd::{chown, ftruncate, mkdir, mkfifo, truncate, unlink, User},
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
    fs::File,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    utils::{chmod, lchmod, lchown, link, rename, symlink},
};

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

fn assert_eloop_final<F, T: Debug>(ctx: &mut TestContext, f: F)
where
    F: Fn(&Path) -> nix::Result<T>,
{
    let (p1, p2) = create_loop_symlinks(ctx);

    assert_eq!(f(&p1.join("test")).unwrap_err(), Errno::ELOOP);
    assert_eq!(f(&p2.join("test")).unwrap_err(), Errno::ELOOP);
}

crate::test_case! {eloop}
fn eloop(ctx: &mut TestContext) {
    fn assert_eloop_folder<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let (p1, p2) = create_loop_symlinks(ctx);

        assert_eq!(f(&p1).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p2).unwrap_err(), Errno::ELOOP);
    }

    fn assert_eloop_link<F, T: Debug>(ctx: &mut TestContext, f: F)
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

    fn assert_eloop_all<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eloop_folder(ctx, &f);
        assert_eloop_final(ctx, f);
    }

    // TODO: Add rmdir upstream
    // assert_eloop(ctx, |p| rmdir(p, Mode::empty()));
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
        assert_eloop_final(ctx, |p| chflags(p, FileFlag::empty()));
    }
    // chmod/06.t
    assert_eloop_all(ctx, |p| chmod(p, Mode::empty()));

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        // chmod/06.t#L25
        assert_eloop_final(ctx, |p| lchmod(p, Mode::empty()));
    }

    // chown/06.t
    assert_eloop_all(ctx, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    // chown/06.t#L25
    assert_eloop_final(ctx, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    // link/08.t
    assert_eloop_link(ctx, |p1, p2| link(p1, p2));
    // mkdir/07.t
    assert_eloop_final(ctx, |p| mkdir(p, Mode::empty()));
    // mkfifo/07.t
    assert_eloop_final(ctx, |p| mkfifo(p, Mode::empty()));
    // mknod/07.t
    assert_eloop_final(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    // open/12.t
    assert_eloop_final(ctx, |p| open(p, OFlag::empty(), Mode::empty()));
    //TODO: open/16.t
    // reanme/11.t
    assert_eloop_link(ctx, |p1, p2| rename(p1, p2));
    // symlink/07.t
    assert_eloop_final(ctx, |p| symlink(Path::new("test"), p));
    // truncate/07.t
    assert_eloop_final(ctx, |p| truncate(p, 0));
    // unlink/07.t
    assert_eloop_final(ctx, |p| unlink(p));
}

crate::test_case! {enotdir => [Regular, Fifo, Block, Char, Socket]}
fn enotdir(ctx: &mut TestContext, ft: FileType) {
    /// Asserts that it returns ENOTDIR if a component of the path prefix is not a directory.
    fn assert_enotdir<T: Debug, F>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let base_path = ctx.create(ft.clone()).unwrap();
        let path = base_path.join("previous_not_dir");

        assert_eq!(f(&path).unwrap_err(), Errno::ENOTDIR);
    }

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

    /// Asserts that it returns ENOTDIR when the 'from' argument is a directory, but 'to' is not a directory.
    fn assert_enotdir_from_to<T: Debug, F>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let path = ctx.create(ft.clone()).unwrap();
        let dir = ctx.create(FileType::Dir).unwrap();

        assert_eq!(f(&dir, &path).unwrap_err(), Errno::ENOTDIR);
    }

    // TODO: Add rmdir upstream
    // assert_enotdir(&path, |p| rmdir(p, Mode::empty()));
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
    assert_enotdir(ctx, &ft, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enotdir(ctx, &ft, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    // link/01.t
    assert_enotdir_two_params(ctx, &ft, |p1, p2| link(p1, p2));
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
    // rename/12.t
    assert_enotdir_two_params(ctx, &ft, |p1, p2| rename(p1, p2));
    // rename/13.t
    assert_enotdir_from_to(ctx, &ft, |from, to| rename(from, to));
    // symlink/01.t
    assert_enotdir(ctx, &ft, |p| symlink(Path::new("test"), p));
    // (f)truncate/01.t
    assert_enotdir(ctx, &ft, |p| truncate(p, 0));
    // unlink/01.t
    assert_enotdir(ctx, &ft, |p| unlink(p));
}

crate::test_case! {enotdir_privileged, root => [Regular, Fifo, Block, Char, Socket]}
fn enotdir_privileged(ctx: &mut TestContext, ft: FileType) {
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

crate::test_case! {enoent}
fn enoent(ctx: &mut TestContext) {
    fn create_fake_path(ctx: &mut TestContext) -> PathBuf {
        let dir = ctx.create(FileType::Dir).unwrap();
        dir.join("not_existent")
    }

    /// Asserts that it returns ENOENT if the named file does not exist
    fn assert_enoent<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let fake_path = create_fake_path(ctx);
        assert_eq!(f(&fake_path).unwrap_err(), Errno::ENOENT);
    }

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

    /// Asserts that it returns ENOENT if a component of the path prefix does not exist
    fn assert_enoent_final_comp<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let fake_path = create_fake_path(ctx);
        assert_eq!(f(&fake_path.join("test")).unwrap_err(), Errno::ENOENT);
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
    assert_enoent_final_comp(ctx, |p| chmod(p, Mode::empty()));
    assert_enoent_link(ctx, |p| chmod(p, Mode::empty()));

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        // chmod/04.t
        assert_enoent(ctx, |p| lchmod(p, Mode::empty()));
    }

    // chown/04.t
    assert_enoent(ctx, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent_final_comp(ctx, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent_link(ctx, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent(ctx, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });

    // link/04.t
    assert_enoent_two_params(ctx, |p1, p2| link(p1, p2));
    //TODO: DO link/09.t?
    // mkdir/04.t
    assert_enoent_final_comp(ctx, |p| mkdir(p, Mode::empty()));
    // mknod/04.t
    assert_enoent_final_comp(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    // mkfifo/04.t
    assert_enoent_final_comp(ctx, |p| mkfifo(p, Mode::empty()));
    // open/04.t
    assert_enoent(ctx, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    assert_enoent_final_comp(ctx, |p| open(p, OFlag::O_CREAT, Mode::S_IRWXU));
    assert_enoent(ctx, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    // rename/03.t
    assert_enoent_two_params(ctx, |p1, p2| rename(p1, p2));
    // symlink/04.t
    assert_enoent_final_comp(ctx, |p| symlink(Path::new("test"), p));
    // (f)truncate/04.t
    assert_enoent(ctx, |p| truncate(p, 0));
    // unlink/04.t
    assert_enoent(ctx, |p| unlink(p));
}

crate::test_case! {eacces, serialized, root}
fn eacces(ctx: &mut SerializedTestContext) {
    /// Asserts that it returns EACCES when search permission is denied for a component of the path prefix
    fn assert_eacces_search_perm<F, T: Debug>(ctx: &mut SerializedTestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.create(FileType::Dir).unwrap();
        let user = User::from_name("nobody").unwrap().unwrap();
        let mode = Mode::from_bits_truncate(0o644);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();

        let f = AssertUnwindSafe(f);
        ctx.as_user(Some(user.uid), Some(user.gid), || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    /// Asserts that it returns EACCES when write permission is denied on the parent directory of the directory to be created
    fn assert_eacces_write_perm<F, T: Debug>(ctx: &mut SerializedTestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.create(FileType::Dir).unwrap();
        let user = User::from_name("nobody").unwrap().unwrap();
        let mode = Mode::from_bits_truncate(0o555);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();

        let f = AssertUnwindSafe(f);
        ctx.as_user(Some(user.uid), Some(user.gid), || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    /// Asserts that it returns EACCES if the named file is not writable by the user
    fn assert_eacces_write_perm_file<F, T: Debug>(ctx: &mut SerializedTestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.create(FileType::Dir).unwrap();
        let user = User::from_name("nobody").unwrap().unwrap();
        let mode = Mode::from_bits_truncate(0o444);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();

        let f = AssertUnwindSafe(f);
        ctx.as_user(Some(user.uid), Some(user.gid), || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
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
        // chflags/05.t
        assert_eacces_search_perm(ctx, |p| chflags(p, FileFlag::empty()));
    }

    // chmod/05.t
    assert_eacces_search_perm(ctx, |p| chmod(p, Mode::empty()));
    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        assert_eacces_search_perm(ctx, |p| lchmod(p, Mode::empty()));
    }

    // chown/05.t
    // TODO: Blocked by #63
    // assert_eacces(ctx, |p| {
    //     chown(
    //         p,
    //         Some(User::from_name("nobody").unwrap().unwrap().uid),
    //         None,
    //     )
    // });
    // assert_eacces(ctx, |p| {
    //     lchown(
    //         p,
    //         Some(User::from_name("nobody").unwrap().unwrap().uid),
    //         None,
    //     )
    // });
    // TODO: link specialization
    // mkdir/05.t
    assert_eacces_search_perm(ctx, |p| mkdir(p, Mode::empty()));
    // mkdir/06.t
    assert_eacces_write_perm(ctx, |p| mkdir(p, Mode::empty()));
    // mkfifo/05.t
    assert_eacces_search_perm(ctx, |p| mkfifo(p, Mode::empty()));
    // mkfifo/06.t
    assert_eacces_write_perm(ctx, |p| mkfifo(p, Mode::empty()));
    // mknod/05.t
    assert_eacces_search_perm(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    // mknod/06.t
    assert_eacces_write_perm(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    //TODO: open
    assert_eacces_search_perm(ctx, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    assert_eacces_write_perm(ctx, |p| {
        open(p, OFlag::O_CREAT | OFlag::O_RDONLY, Mode::empty())
    });
    //TODO: rename
    // symlink/05.t
    assert_eacces_search_perm(ctx, |p| symlink(Path::new("test"), p));
    // symlink/06.t
    assert_eacces_write_perm(ctx, |p| symlink(Path::new("test"), p));
    // (f)truncate/05.t
    assert_eacces_write_perm_file(ctx, |p| truncate(p, 0));
    // (f)truncate/06.t
    assert_eacces_search_perm(ctx, |p| truncate(p, 0));
    // unlink/05.t
    assert_eacces_search_perm(ctx, |p| unlink(p));
    // unlink/06.t
    assert_eacces_search_perm(ctx, |p| unlink(p));
}

/// Asserts that it returns EEXIST if the named file exists
fn assert_eexist<F, T: Debug>(ctx: &mut TestContext, ft: &FileType, f: F)
where
    F: Fn(&Path) -> nix::Result<T>,
{
    let path = ctx.create(ft.clone()).unwrap();
    assert_eq!(f(&path).unwrap_err(), Errno::EEXIST);
}

crate::test_case! {eexist => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]}
fn eexist(ctx: &mut TestContext, ft: FileType) {
    /// Asserts that it returns EEXIST or ENOTEMPTY if the 'to' argument is a directory and is not empty
    //TODO: Add rmdir/{06,12}.t
    fn assert_eexist_enotempty<F, T: Debug>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let from_dir = ctx.create(FileType::Dir).unwrap();
        let to_dir = ctx.create(FileType::Dir).unwrap();
        ctx.create_named(ft.clone(), to_dir.join("test")).unwrap();
        assert!(matches!(
            f(&from_dir, &to_dir).unwrap_err(),
            Errno::EEXIST | Errno::ENOTEMPTY
        ));
    }

    let default_mode = Mode::from_bits_truncate(0o644);

    // mkdir/10.t
    assert_eexist(ctx, &ft, |p| mkdir(p, Mode::empty()));
    assert_eexist_enotempty(ctx, &ft, |from, to| rename(from, to));
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

    // TODO:rmdir
    // assert_eexist_enotempty(ctx, &ft, |from, to| rmdir(from, to));
    // symlink/08.t
    assert_eexist(ctx, &ft, |p| symlink(&*PathBuf::from("test"), p));
}

crate::test_case! {eexist_privileged, root => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]}
fn eexist_privileged(ctx: &mut TestContext, ft: FileType) {
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
}

crate::test_case! {efault}
fn efault(_ctx: &mut TestContext) {}

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

    /// open may return EINVAL when an attempt was made to open a descriptor with an illegal combination of O_RDONLY, O_WRONLY, and O_RDWR
    fn assert_einval_open(ctx: &mut TestContext, flags: OFlag) {
        let path = ctx.create(FileType::Regular).unwrap();
        assert!(matches!(
            open(&path, flags, Mode::empty()),
            Ok(_) | Err(Errno::EINVAL)
        ));
    }

    // open/23.t
    assert_einval_open(ctx, OFlag::O_RDONLY | OFlag::O_RDWR);
    assert_einval_open(ctx, OFlag::O_WRONLY | OFlag::O_RDWR);
    assert_einval_open(ctx, OFlag::O_RDONLY | OFlag::O_WRONLY | OFlag::O_RDWR);

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

    //TODO: rmdir/12.t
}

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

crate::test_case! {enametoolong}
fn enametoolong(ctx: &mut TestContext) {
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

    /// Asserts that it returns ENAMETOOLONG if an entire of either path name exceeded {PATH_MAX} characters
    fn assert_enametoolong_path_two_params<F, T: Debug + std::cmp::PartialEq>(
        ctx: &mut TestContext,
        f: F,
    ) where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let mut invalid_path = ctx.create_path_max(FileType::Regular).unwrap();
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

    //TODO: lchflags too?
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios"
    ))]
    {
        // chflags/02.t
        assert_enametoolong_comp(ctx, |p| chflags(p, FileFlag::empty()));
        // chflags/03.t
        assert_enametoolong_path(ctx, |p| chflags(p, FileFlag::empty()));
    }

    // chmod/02.t
    assert_enametoolong_comp(ctx, |p| chmod(p, Mode::empty()));
    // chmod/03.t
    assert_enametoolong_path(ctx, |p| chmod(p, Mode::empty()));
    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    {
        assert_enametoolong_comp(ctx, |p| lchmod(p, Mode::empty()));
        assert_enametoolong_path(ctx, |p| lchmod(p, Mode::empty()));
    }

    // chown/02.t
    // chown/03.t
    let user = User::from_name("nobody").unwrap().unwrap();
    assert_enametoolong_comp(ctx, |p| chown(p, Some(user.uid), Some(user.gid)));
    assert_enametoolong_path(ctx, |p| chown(p, Some(user.uid), Some(user.gid)));
    assert_enametoolong_comp(ctx, |p| lchown(p, Some(user.uid), Some(user.gid)));
    assert_enametoolong_path(ctx, |p| lchown(p, Some(user.uid), Some(user.gid)));

    // link/02.t
    assert_enametoolong_comp_two_params(ctx, |p1, p2| link(p1, p2));
    // link/03.t
    assert_enametoolong_path_two_params(ctx, |p1, p2| link(p1, p2));

    // mkfifo/02.t
    assert_enametoolong_comp(ctx, |p| mkfifo(p, Mode::empty()));
    // mkfifo/03.t
    assert_enametoolong_path(ctx, |p| mkfifo(p, Mode::empty()));

    // mkdir/02.t
    assert_enametoolong_comp(ctx, |p| mkdir(p, Mode::empty()));
    // mkdir/03.t
    assert_enametoolong_path(ctx, |p| mkdir(p, Mode::empty()));

    // mknod/02.t
    assert_enametoolong_comp(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    // mknod/03.t
    assert_enametoolong_path(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));

    // open/02.t
    assert_enametoolong_comp(ctx, |p| open(p, OFlag::O_CREAT, Mode::empty()));
    // open/03.t
    assert_enametoolong_path(ctx, |p| open(p, OFlag::O_CREAT, Mode::empty()));

    // rename/01.t
    assert_enametoolong_comp_two_params(ctx, |p1, p2| rename(p1, p2));
    // rename/02.t
    assert_enametoolong_path_two_params(ctx, |p1, p2| rename(p1, p2));

    // (f)truncate/02.t
    assert_enametoolong_comp(ctx, |p| truncate(p, 0));
    // (f)truncate/03.t
    assert_enametoolong_path(ctx, |p| truncate(p, 0));

    // symlink/02.t
    let valid_path = ctx.create(FileType::Regular).unwrap();
    assert_enametoolong_comp(ctx, |p| symlink(&*valid_path, p));
    // symlink/03.t
    assert_enametoolong_path_two_params(ctx, |p1, p2| symlink(p1, p2));

    // unlink/02.t
    assert_enametoolong_comp(ctx, |p| unlink(p));
    // unlink/03.t
    assert_enametoolong_path(ctx, |p| unlink(p));

    //TODO: rmdir
}

crate::test_case! {enametoolong_privileged, root}
fn enametoolong_privileged(ctx: &mut TestContext) {
    // mknod/02.t
    assert_enametoolong_comp(ctx, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
    assert_enametoolong_comp(ctx, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
    // mknod/03.t
    assert_enametoolong_path(ctx, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
    assert_enametoolong_path(ctx, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
}

crate::test_case! {etxtbsy}
fn etxtbsy(ctx: &mut TestContext) {
    fn assert_etxtbsy<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let sleep_path =
            String::from_utf8(Command::new("which").arg("sleep").output().unwrap().stdout).unwrap();
        let sleep_path = sleep_path.trim();
        let exec_path = ctx.base_path().join("sleep");
        std::fs::copy(sleep_path, &exec_path).unwrap();
        let mut sleep_process = Command::new(&exec_path).arg("5").spawn().unwrap();
        assert_eq!(f(&exec_path).unwrap_err(), Errno::ETXTBSY);
        sleep_process.kill().unwrap();
    }

    // open/20.t
    assert_etxtbsy(ctx, |p| open(p, OFlag::O_WRONLY, Mode::empty()));
    assert_etxtbsy(ctx, |p| open(p, OFlag::O_RDWR, Mode::empty()));
    assert_etxtbsy(ctx, |p| {
        open(p, OFlag::O_RDONLY | OFlag::O_TRUNC, Mode::empty())
    });
    // (f)truncate/11.t
    assert_etxtbsy(ctx, |p| truncate(p, 123));
}
