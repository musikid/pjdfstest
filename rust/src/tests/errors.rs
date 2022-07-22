use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, Mode, SFlag},
    unistd::{chown, mkdir, mkfifo, truncate, unlink, Uid, User},
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
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    utils::{chmod, lchmod, lchown, link, rename, symlink},
};

crate::test_case! {eloop}
fn eloop(ctx: &mut TestContext) {
    let p1 = ctx
        .create_named(FileType::Symlink(Some(PathBuf::from("loop2"))), "loop1")
        .unwrap();
    let p2 = ctx
        .create_named(FileType::Symlink(Some(PathBuf::from("loop1"))), "loop2")
        .unwrap();
    let p3 = ctx.create(FileType::Regular).unwrap();

    fn assert_eloop_folder<F, T: Debug>(p1: &Path, p2: &Path, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eq!(f(p1).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(p2).unwrap_err(), Errno::ELOOP);
    }
    fn assert_eloop_final<F, T: Debug>(p1: &Path, p2: &Path, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eq!(f(&p1.join("test")).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p2.join("test")).unwrap_err(), Errno::ELOOP);
    }
    fn assert_eloop_link<F, T: Debug>(p1: &Path, p2: &Path, p3: &Path, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        assert_eq!(f(&p1.join("test"), &p3).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p2.join("test"), &p3).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p3, &p1.join("test")).unwrap_err(), Errno::ELOOP);
        assert_eq!(f(&p3, &p2.join("test")).unwrap_err(), Errno::ELOOP);
    }
    fn assert_eloop_all<F, T: Debug>(p1: &Path, p2: &Path, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eloop_folder(p1, p2, &f);
        assert_eloop_final(p1, p2, f);
    }

    // TODO: Add rmdir upstream
    // assert_eloop(&p1, &p2, |p| rmdir(p, Mode::empty()));
    assert_eloop_final(&p1, &p2, |p| chflags(p, FileFlag::empty()));
    assert_eloop_all(&p1, &p2, |p| chmod(p, Mode::empty()));
    assert_eloop_final(&p1, &p2, |p| lchmod(p, Mode::empty()));
    assert_eloop_all(&p1, &p2, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_eloop_final(&p1, &p2, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_eloop_link(&p1, &p2, &p3, |p1, p2| link(p1, p2));
    assert_eloop_final(&p1, &p2, |p| mkdir(p, Mode::empty()));
    assert_eloop_final(&p1, &p2, |p| mkfifo(p, Mode::empty()));
    assert_eloop_final(&p1, &p2, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
    assert_eloop_final(&p1, &p2, |p| open(p, OFlag::empty(), Mode::empty()));
    assert_eloop_link(&p1, &p2, &p3, |p1, p2| rename(p1, p2));
    assert_eloop_final(&p1, &p2, |p| symlink(Path::new("test"), p));
    assert_eloop_final(&p1, &p2, |p| truncate(p, 0));
    assert_eloop_final(&p1, &p2, |p| unlink(p));
}

crate::test_case! {enotdir => [Regular, Fifo, Block, Char, Socket]}
fn enotdir(ctx: &mut TestContext, ft: FileType) {
    let base_path = ctx.create(ft).unwrap();
    let path = base_path.join("previous_not_dir");
    let dir = ctx.create(FileType::Dir).unwrap();

    fn assert_enotdir<T: Debug, F>(path: &Path, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eq!(f(path).unwrap_err(), Errno::ENOTDIR);
    }

    // TODO: Add rmdir upstream
    // assert_enotdir(&path, |p| rmdir(p, Mode::empty()));
    assert_enotdir(&path, |p| chflags(p, FileFlag::empty()));
    assert_enotdir(&path, |p| chmod(p, Mode::empty()));
    assert_enotdir(&path, |p| lchmod(p, Mode::empty()));
    assert_enotdir(&path, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enotdir(&path, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enotdir(&path, |p| link(p, &base_path));
    assert_enotdir(&path, |p| link(&*base_path, p));
    assert_enotdir(&path, |p| mkdir(p, Mode::empty()));
    assert_enotdir(&path, |p| mkfifo(p, Mode::empty()));
    assert_enotdir(&path, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
    assert_enotdir(&path, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    assert_enotdir(&path, |p| {
        open(p, OFlag::O_CREAT, Mode::from_bits_truncate(0o644))
    });
    assert_enotdir(&path, |p| rename(&dir, &base_path));
    assert_enotdir(&path, |p| rename(p, Path::new("test")));
    assert_enotdir(&path, |p| rename(&*base_path, p));
    assert_enotdir(&path, |p| symlink(Path::new("test"), p));
    assert_enotdir(&path, |p| truncate(p, 0));
    assert_enotdir(&path, |p| unlink(p));
}

crate::test_case! {enoent}
fn enoent(ctx: &mut TestContext) {
    let dir = ctx.create(FileType::Dir).unwrap();
    let fake_path = dir.join("not_existent");
    let real_path = ctx.create(FileType::Regular).unwrap();
    let link_to_fake_path = ctx
        .create(FileType::Symlink(Some(fake_path.to_path_buf())))
        .unwrap();

    fn assert_enoent<F, T: Debug>(path: &Path, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eq!(f(path).unwrap_err(), Errno::ENOENT);
    }

    fn assert_enoent_final_comp<F, T: Debug>(path: &Path, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        assert_eq!(f(&path.join("test")).unwrap_err(), Errno::ENOENT);
    }

    fn assert_enoent_two_params<F, T: Debug>(fake_path: &Path, real_path: &Path, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        assert_eq!(
            f(&fake_path.join("test"), real_path).unwrap_err(),
            Errno::ENOENT
        );
        assert_eq!(
            f(real_path, &fake_path.join("test")).unwrap_err(),
            Errno::ENOENT
        );
    }

    assert_enoent(&fake_path, |p| chflags(p, FileFlag::empty()));
    assert_enoent(&fake_path, |p| chmod(p, Mode::empty()));
    assert_enoent_final_comp(&fake_path, |p| chmod(p, Mode::empty()));
    assert_enoent(&link_to_fake_path, |p| chmod(p, Mode::empty()));
    assert_enoent(&fake_path, |p| lchmod(p, Mode::empty()));
    assert_enoent_final_comp(&fake_path, |p| chmod(p, Mode::empty()));
    assert_enoent(&fake_path, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent_final_comp(&fake_path, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent(&link_to_fake_path, |p| {
        chown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent(&fake_path, |p| {
        lchown(
            p,
            Some(User::from_name("nobody").unwrap().unwrap().uid),
            None,
        )
    });
    assert_enoent_two_params(&fake_path, &real_path, |p1, p2| link(p1, p2));
    //TODO: DO link/09.t?
    assert_enoent_final_comp(&fake_path, |p| mkdir(p, Mode::empty()));
    assert_enoent_final_comp(&fake_path, |p| mkfifo(p, Mode::empty()));
    assert_enoent_final_comp(&fake_path, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
    assert_enoent(&fake_path, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    assert_eq!(
        open(&fake_path.join("test"), OFlag::O_CREAT, Mode::S_IRWXU).unwrap_err(),
        Errno::ENOENT
    );
    assert_eq!(
        open(&fake_path, OFlag::O_RDONLY, Mode::empty()).unwrap_err(),
        Errno::ENOENT
    );
    assert_enoent_two_params(&fake_path, &real_path, |p1, p2| rename(p1, p2));
    assert_enoent_final_comp(&fake_path, |p| symlink(Path::new("test"), p));
    assert_enoent(&fake_path, |p| truncate(p, 0));
    assert_enoent(&fake_path, |p| unlink(p));
}

crate::test_case! {eacces, serialized}
fn eacces(ctx: &mut SerializedTestContext) {
    /// Asserts that it returns EACCES when search permission is denied for a component of the path prefix
    fn assert_eacces_search_perm<F, T: Debug>(ctx: &mut SerializedTestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let dir = ctx.create(FileType::Dir).unwrap();
        let user = User::from_name("nobody").unwrap().unwrap();
        let mode = Mode::from_bits_truncate(0o644);

        let f = AssertUnwindSafe(f);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();
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

        let f = AssertUnwindSafe(f);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();
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

        let f = AssertUnwindSafe(f);

        let path = dir.join("test");
        chmod(&dir, mode).unwrap();
        ctx.as_user(Some(user.uid), Some(user.gid), || {
            assert_eq!(f(&path).unwrap_err(), Errno::EACCES);
        });
    }

    assert_eacces_search_perm(ctx, |p| chflags(p, FileFlag::empty()));
    assert_eacces_search_perm(ctx, |p| chmod(p, Mode::empty()));
    assert_eacces_search_perm(ctx, |p| lchmod(p, Mode::empty()));
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
    assert_eacces_write_perm(ctx, |p| mkdir(p, Mode::empty()));
    assert_eacces_search_perm(ctx, |p| mkdir(p, Mode::empty()));
    assert_eacces_search_perm(ctx, |p| mkfifo(p, Mode::empty()));
    assert_eacces_write_perm(ctx, |p| mkfifo(p, Mode::empty()));
    assert_eacces_search_perm(ctx, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    //TODO: open
    assert_eacces_search_perm(ctx, |p| open(p, OFlag::O_RDONLY, Mode::empty()));
    assert_eacces_write_perm(ctx, |p| open(p, OFlag::O_CREAT, Mode::empty()));
    //TODO: rename
    assert_eacces_write_perm(ctx, |p| symlink(Path::new("test"), p));
    assert_eacces_search_perm(ctx, |p| symlink(Path::new("test"), p));
    assert_eacces_write_perm_file(ctx, |p| truncate(p, 0));
    assert_eacces_search_perm(ctx, |p| truncate(p, 0));
    assert_eacces_search_perm(ctx, |p| unlink(p));
    assert_eacces_search_perm(ctx, |p| unlink(p));
}

crate::test_case! {eexist => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]}
fn eexist(ctx: &mut TestContext, ft: FileType) {
    fn assert_eexist<F, T: Debug>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let path = ctx.create(ft.clone()).unwrap();
        assert_eq!(f(&path).unwrap_err(), Errno::EEXIST);
    }

    /// Asserts that it returns EEXIST or ENOTEMPTY
    fn assert_eexist_enotempty<F, T: Debug>(ctx: &mut TestContext, ft: &FileType, f: F)
    where
        F: Fn(&Path, &Path) -> nix::Result<T>,
    {
        let from_dir = ctx.create(FileType::Dir).unwrap();
        let to_dir = ctx.create(FileType::Dir).unwrap();
        ctx.create_named(ft.clone(), to_dir.join("test")).unwrap();
        assert!(match f(&from_dir, &to_dir).unwrap_err() {
            Errno::EEXIST | Errno::ENOTEMPTY => true,
            _ => false,
        });
    }

    let default_mode = Mode::from_bits_truncate(0o644);

    assert_eexist(ctx, &ft, |p| mkdir(p, Mode::empty()));
    assert_eexist_enotempty(ctx, &ft, |from, to| rename(from, to));
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFBLK, Mode::empty(), 0));
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFCHR, Mode::empty(), 0));
    assert_eexist(ctx, &ft, |p| mknod(p, SFlag::S_IFIFO, Mode::empty(), 0));
    assert_eexist(ctx, &ft, |p| {
        open(p, OFlag::O_CREAT | OFlag::O_EXCL, default_mode)
    });
    assert_eexist(ctx, &ft, |p| mkfifo(p, default_mode));

    let regular_file = ctx.create(FileType::Regular).unwrap();
    assert_eexist(ctx, &ft, |p| link(&*regular_file, p));

    // TODO:rmdir
    // assert_eexist_enotempty(ctx, &ft, |from, to| rmdir(from, to));
    assert_eexist(ctx, &ft, |p| symlink(&*PathBuf::from("test"), p));
}

crate::test_case! {efault}
fn efault(ctx: &mut TestContext) {}

crate::test_case! {enametoolong_comp_max}
fn enametoolong_comp_max(ctx: &mut TestContext) {
    let path = ctx.create_name_max(FileType::Regular).unwrap();

    fn assert_enametoolong<F, T: Debug + std::cmp::PartialEq>(
        invalid_path: &Path,
        valid_path: &Path,
        expected: T,
        f: F,
    ) where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let res = f(invalid_path);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
        assert_eq!(f(invalid_path).unwrap_err(), Errno::ENOTDIR);
    }
}
