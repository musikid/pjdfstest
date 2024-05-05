use std::{
    collections::HashSet,
    fmt::Debug,
    fs::metadata,
    os::{
        freebsd::fs::MetadataExt as _,
        unix::prelude::{FileTypeExt, MetadataExt as _},
    },
    path::Path,
    sync::atomic::AtomicBool,
};

use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, mode_t, stat, FileFlag, Mode, SFlag},
    unistd::{chflags, close, mkdir, mkfifo, truncate, unlink},
};

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
use crate::utils::lchmod;

use crate::{
    context::{FileType, TestContext},
    flags::FileFlags,
    test::FileSystemFeature,
    utils::{chmod, link, rename, rmdir, symlink, ALLPERMS},
};

/// Return flags which intersects with the provided ones
/// and those available in the configuration,
/// along with the other available in the configuration (representing the flags which don't trigger errors in this context).
pub fn get_flags_intersection(
    supported_flags: &HashSet<FileFlags>,
    error_flags: &[FileFlags],
) -> (Vec<FileFlags>, Vec<FileFlags>) {
    let error_flags: HashSet<_> = error_flags.iter().copied().collect();
    let supported_err_flags: HashSet<_> = supported_flags
        .intersection(&error_flags)
        .copied()
        .collect();
    let valid_flags: Vec<_> = supported_flags
        .difference(&supported_err_flags)
        .copied()
        .collect();

    (supported_err_flags.into_iter().collect(), valid_flags)
}

/// Assert that setting `flags` on the file's parent directory if `parent` is `true`
/// or the file itself otherwise do make the function fail with EPERM.
/// Also assert that `valid_flags` do not make the function fail.
/// The `check` function should retuns a `bool` which should succeed when the tested function succeed.
/// If the file needs to be created before, a [`FileType`](crate::runner::context::FileType) should be provided.
fn assert_flags<T: Debug, F, C>(
    ctx: &TestContext,
    flags: &[FileFlags],
    valid_flags: &[FileFlags],
    parent: bool,
    created_type: Option<FileType>,
    mut f: F,
    check: C,
) where
    F: FnMut(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let get_files = || {
        let (flagged_file, file) = if parent {
            let dir = ctx.create(FileType::Dir).unwrap();
            let file = dir.join("file");
            let file = if let Some(created_type) = created_type.clone() {
                ctx.new_file(created_type).name(file).create().unwrap()
            } else {
                file
            };

            (dir, file)
        } else {
            let path = ctx.gen_path();
            let file = if let Some(created_type) = created_type.clone() {
                ctx.new_file(created_type).name(path).create().unwrap()
            } else {
                path
            };

            (file.clone(), file)
        };

        (flagged_file, file)
    };

    for &flag in flags {
        let raw_flag: FileFlag = flag.into();
        let (flagged_file, file) = get_files();

        chflags(&flagged_file, raw_flag).unwrap();

        let set_flags = metadata(&flagged_file).unwrap().st_flags();
        assert!(
            set_flags as u64 & raw_flag.bits() > 0,
            "File should have {flag} set but only have {set_flags}"
        );

        assert!(
            matches!(f(&file), Err(Errno::EPERM)),
            "{flag} does not trigger EPERM"
        );
        assert!(!check(&file), "Error file check failed for {flag}");

        chflags(&flagged_file, FileFlag::empty()).unwrap();

        assert!(
            f(&file).is_ok(),
            "Failure when checking when unsetting flag {flag}"
        );
        assert!(check(&file), "Success file check failed for {flag}");
    }

    for &flag in valid_flags {
        let raw_flag: FileFlag = flag.into();
        let (flagged_file, file) = get_files();

        chflags(&flagged_file, raw_flag).unwrap();

        assert!(
            f(&file).is_ok(),
            "Failure when checking if syscall is working for valid flag {flag}"
        );
        assert!(
            check(&file),
            "Success file check failed for valid flag {flag}"
        );
    }
}

/// Specialization of [`assert_flags`] for named files.
fn assert_flags_named_file<T: Debug, F, C>(
    ctx: &TestContext,
    flags: &[FileFlags],
    valid_flags: &[FileFlags],
    created_type: Option<FileType>,
    f: F,
    check: C,
) where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    assert_flags(ctx, flags, valid_flags, false, created_type, f, check)
}

/// Specialization of [`assert_flags`] for parent directory.
fn assert_flags_parent<T: Debug, F, C>(
    ctx: &TestContext,
    flags: &[FileFlags],
    valid_flags: &[FileFlags],
    created_type: Option<FileType>,
    f: F,
    check: C,
) where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    assert_flags(ctx, flags, valid_flags, true, created_type, f, check)
}

crate::test_case! {
    /// open returns EPERM when the named file has its immutable flag set
    /// and the file is to be modified
    // open/10.t
    immutable_file, FileSystemFeature::Chflags
}
fn immutable_file(ctx: &mut TestContext) {
    let immutable_flags = FileFlags::IMMUTABLE_FLAGS.iter().copied().collect();
    let flags: Vec<_> = ctx
        .features_config()
        .file_flags
        .intersection(&immutable_flags)
        .copied()
        .collect();
    let valid_flags = FileFlags::UNDELETABLE_FLAGS.iter().copied().collect();
    let valid_flags: Vec<_> = ctx
        .features_config()
        .file_flags
        .intersection(&valid_flags)
        .copied()
        .collect();

    // TODO: Improve check function
    let state = AtomicBool::new(false);
    let created_type = Some(FileType::Regular);
    let f = |path: &_| {
        let res = open(path, OFlag::O_RDONLY | OFlag::O_TRUNC, Mode::empty()).and_then(close);
        state.store(
            match res {
                Ok(_) => true,
                _ => false,
            },
            std::sync::atomic::Ordering::Relaxed,
        );
        res
    };
    let check = |_: &_| state.load(std::sync::atomic::Ordering::Relaxed);

    assert_flags(ctx, &flags, &valid_flags, false, created_type, f, check)
}

crate::test_case! {
    append_file, FileSystemFeature::Chflags
}
fn append_file(ctx: &mut TestContext) {
    let (flags, _) = get_flags_intersection(
        &ctx.features_config().file_flags,
        FileFlags::APPEND_ONLY_FLAGS,
    );

    // open/11.t
    assert_flags_named_file(
        ctx,
        &flags,
        &[],
        Some(FileType::Regular),
        |path| {
            open(path, OFlag::O_WRONLY, Mode::empty())
                .and_then(|fd| nix::unistd::write(fd, "data".as_bytes()).map(|_| fd))
                .and_then(close)
        },
        |path| {
            let actual_size = metadata(path).unwrap().size();
            actual_size > 0
        },
    );
}

crate::test_case! {
    /// Return EPERM if the parent directory of the named file has its immutable flag set
    immutable_append_file, FileSystemFeature::Chflags
}
fn immutable_append_file(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_flags_intersection(
        &ctx.features_config().file_flags,
        &[FileFlags::IMMUTABLE_FLAGS, FileFlags::APPEND_ONLY_FLAGS].concat(),
    );

    // chmod/08.t
    let mode = Mode::from_bits_truncate(0o100);
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |path| chmod(path, mode),
        |path| metadata(path).map_or(false, |m| m.mode() as mode_t & ALLPERMS == mode.bits()),
    );

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |path| lchmod(path, mode),
        |path| metadata(path).map_or(false, |m| m.mode() as mode_t & ALLPERMS == mode.bits()),
    );

    // (f)truncate/08.t
    // TODO: Failure on ZFS with SF_APPEND
    let size = 123;
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |path| truncate(path, size),
        |path| stat(path).map_or(false, |s| s.st_size == size),
    );

    // link/12.t
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |src| {
            let dest = ctx.gen_path();
            link(src, &*dest)
        },
        |src| metadata(src).map_or(false, |m| m.nlink() == 2),
    );

    // open/10.t
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |path| {
            open(path, OFlag::O_WRONLY, Mode::empty())
                .and_then(|fd| nix::unistd::write(fd, "data".as_bytes()).map(|_| fd))
                .and_then(close)
        },
        |path| {
            let size = metadata(path).unwrap().len();
            size > 0
        },
    );
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |path| {
            open(path, OFlag::O_RDWR, Mode::empty())
                .and_then(|fd| nix::unistd::write(fd, "data".as_bytes()).map(|_| fd))
                .and_then(close)
        },
        |path| {
            let size = metadata(path).unwrap().len();
            size > 0
        },
    );
}

crate::test_case! {
    /// Return EPERM if the parent directory of the named file has its immutable flag set
    immutable_append_undeletable_file, FileSystemFeature::Chflags
}
fn immutable_append_undeletable_file(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_flags_intersection(
        &ctx.features_config().file_flags,
        &[
            FileFlags::IMMUTABLE_FLAGS,
            FileFlags::APPEND_ONLY_FLAGS,
            FileFlags::UNDELETABLE_FLAGS,
        ]
        .concat(),
    );

    // unlink/09.t
    // TODO: Failure on ZFS with SF_APPEND
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        unlink,
        |path| !path.exists(),
    );

    // rmdir/09.t
    // TODO: Failure on ZFS with SF_APPEND
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Dir),
        rmdir,
        |path| !path.exists(),
    );

    // rename/06.t
    // TODO: Failure on ZFS with SF_APPEND
    // TODO: Missing multiple file types
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |from| {
            let to = ctx.gen_path();
            rename(from, &*to)
        },
        |from| !from.exists(),
    );
}

crate::test_case! {
    /// Return EPERM if the parent directory of the named file has its immutable flag set
    immutable_append_parent, FileSystemFeature::Chflags
}
fn immutable_append_parent(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_flags_intersection(
        &ctx.features_config().file_flags,
        &[FileFlags::IMMUTABLE_FLAGS, FileFlags::APPEND_ONLY_FLAGS].concat(),
    );

    // unlink/10.t
    // TODO: Failure on ZFS with SF_APPEND
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        unlink,
        |path| !path.exists(),
    );

    // rename/07.t
    // TODO: Failure on ZFS with SF_APPEND
    // TODO: Missing multiple file types
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Regular),
        |from| {
            let to = ctx.create(FileType::Regular).unwrap();
            rename(from, &to)
        },
        |path| !path.exists(),
    );

    // rmdir/10.t
    // TODO: Failure on ZFS
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        Some(FileType::Dir),
        rmdir,
        |path| !path.exists(),
    );
}

crate::test_case! {
    /// Return EPERM if the parent directory of the named file has its immutable flag set
    immutable_parent, FileSystemFeature::Chflags
}
fn immutable_parent(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_flags_intersection(
        &ctx.features_config().file_flags,
        FileFlags::IMMUTABLE_FLAGS,
    );

    let mode = Mode::from_bits_truncate(0o755);

    // mkdir/08.t
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |path| mkdir(path, mode),
        Path::is_dir,
    );

    // mkfifo/10.t
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |path| mkfifo(path, mode),
        |path| metadata(path).map_or(false, |m| m.file_type().is_fifo()),
    );

    // mknod/09.t
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |path| mknod(path, SFlag::S_IFIFO, mode, 0),
        |path| metadata(path).map_or(false, |m| m.file_type().is_fifo()),
    );

    // open/09.t
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |path| open(path, OFlag::O_RDONLY | OFlag::O_CREAT, mode),
        Path::exists,
    );

    // link/13.t
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |dest| {
            let from = ctx.create(FileType::Regular).unwrap();
            link(&*from, dest)
        },
        |dest| metadata(dest).map_or(false, |m| m.nlink() == 2),
    );

    // rename/08.t
    // TODO: Missing multiple file types
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |to| {
            let from = ctx.create(FileType::Regular).unwrap();
            rename(&*from, to)
        },
        Path::exists,
    );

    // symlink/09.t
    assert_flags_parent(
        ctx,
        &flags,
        &valid_flags,
        None,
        |path| symlink(Path::new("test"), path),
        Path::is_symlink,
    );
}
