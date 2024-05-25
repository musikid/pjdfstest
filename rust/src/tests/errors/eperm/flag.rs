use std::{
    collections::HashSet,
    fmt::Debug,
    fs::metadata,
    os::{
        freebsd::fs::MetadataExt as _,
        unix::prelude::{FileTypeExt, MetadataExt as _},
    },
    path::Path,
};

use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{mknod, mode_t, stat, FileFlag, Mode, SFlag},
    unistd::{chflags, mkdir, mkfifo, truncate, unlink},
};

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
use crate::utils::lchmod;

use crate::{
    config::Config,
    context::{FileType, TestContext},
    flags::FileFlags,
    test::FileSystemFeature,
    utils::{link, rename, rmdir, symlink, ALLPERMS},
};

/// Guard to check whether any of the provided flags is available in the configuration.
pub(crate) fn supports_any_flag_helper(
    flags: &[FileFlags],
    config: &Config,
    _: &Path,
) -> Result<(), anyhow::Error> {
    let flags: HashSet<_> = flags.iter().copied().collect();

    if config.features.file_flags.intersection(&flags).count() == 0 {
        anyhow::bail!("None of the flags used for this test are available in the configuration")
    }

    Ok(())
}

/// Guard to check whether any of the provided flags is available in the configuration.
macro_rules! supports_any_flag {
    (@ $( $flag: expr ),+ $( , )*) => {
        supports_any_flag!(&[ $( $flag ),+ ])
    };

    ($flags: expr) => {
        |config, _p| $crate::tests::errors::eperm::flag::supports_any_flag_helper($flags, config, _p)
    }
}

pub(crate) use supports_any_flag;

/// Return flags which intersects with the provided ones
/// and those available in the configuration,
/// along with the other available in the configuration (representing the flags which don't trigger errors in this context).
pub fn get_supported_and_error_flags(
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
pub(crate) fn assert_flags<T: Debug, F, C>(
    ctx: &TestContext,
    flags: &[FileFlags],
    _valid_flags: &[FileFlags],
    parent: bool,
    created_type: Option<FileType>,
    f: F,
    check: C,
) where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let get_files = || {
        if parent {
            let dir = ctx.create(FileType::Dir).unwrap();
            let path = dir.join("file");
            if let Some(created_type) = created_type.clone() {
                ctx.new_file(created_type).name(&path).create().unwrap();
            }

            (dir, path)
        } else {
            let path = ctx.gen_path();
            if let Some(created_type) = created_type.clone() {
                ctx.new_file(created_type).name(&path).create().unwrap();
            }

            (path.clone(), path)
        }
    };

    for &flag in flags {
        let raw_flag: FileFlag = flag.into();
        let (flagged_file, file) = get_files();

        chflags(&flagged_file, raw_flag).unwrap();

        // TODO: Add flag names list from FileFlag init when nix will be upgraded
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

    // for &flag in valid_flags {
    //     let raw_flag: FileFlag = flag.into();
    //     let (flagged_file, file) = get_files();

    //     chflags(&flagged_file, raw_flag).unwrap();

    //     assert!(
    //         f(&file).is_ok(),
    //         "Failure when checking if syscall is working for valid flag {flag}"
    //     );
    //     assert!(
    //         check(&file),
    //         "Success file check failed for valid flag {flag}"
    //     );
    // }
}

/// Specialization of [`assert_flags`] for named files.
pub(crate) fn assert_flags_named_file<T: Debug, F, C>(
    ctx: &TestContext,
    flags: &[FileFlags],
    valid_flags: &[FileFlags],
    created_type: FileType,
    f: F,
    check: C,
) where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    assert_flags(ctx, flags, valid_flags, false, Some(created_type), f, check)
}

/// Specialization of [`assert_flags`] for parent directory.
pub(crate) fn assert_flags_parent<T: Debug, F, C>(
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
    /// Returns EPERM if the named file has its immutable or append-only flag set
    immutable_append_file, root, FileSystemFeature::Chflags
}
fn immutable_append_file(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_supported_and_error_flags(
        &ctx.features_config().file_flags,
        &[FileFlags::IMMUTABLE_FLAGS, FileFlags::APPEND_ONLY_FLAGS].concat(),
    );

    // (f)truncate/08.t
    // TODO: Failure on ZFS with SF_APPEND
    let size = 123;
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        FileType::Regular,
        |path| truncate(path, size),
        |path| stat(path).map_or(false, |s| s.st_size == size),
    );

    // link/12.t
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        FileType::Regular,
        |src| {
            let dest = ctx.gen_path();
            link(src, &*dest)
        },
        |src| metadata(src).map_or(false, |m| m.nlink() == 2),
    );
}

pub(crate) fn immutable_append_named_helper<T: Debug, F, C>(ctx: &mut TestContext, f: F, check: C)
where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let (flags, valid_flags) = get_supported_and_error_flags(
        &ctx.features_config().file_flags,
        &[FileFlags::IMMUTABLE_FLAGS, FileFlags::APPEND_ONLY_FLAGS].concat(),
    );

    assert_flags_named_file(ctx, &flags, &valid_flags, FileType::Regular, f, check)
}

/// Create a test case which asserts that the syscall returns EPERM
/// if the named file has its immutable or append-only flag set.
///
/// The macro takes as arguments the syscall identifier,
/// the function to produce an output which can be verified and the check
/// function which asserts that it worked using the previously produced result.
macro_rules! immutable_append_named_test_case {
    ($syscall: ident, $f: expr, $c: expr) => {
        $crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EPERM if the named file has its immutable or append-only flag set")]
            immutable_append_named, root;
            crate::tests::errors::eperm::flag::supports_any_flag!($crate::flags::FileFlags::IMMUTABLE_FLAGS),
            crate::tests::errors::eperm::flag::supports_any_flag!($crate::flags::FileFlags::APPEND_ONLY_FLAGS)
        }
        fn immutable_append_named(ctx: &mut crate::context::TestContext) {
            $crate::tests::errors::eperm::flag::immutable_append_named_helper(ctx, $f, $c)
        }
    };
}

pub(crate) use immutable_append_named_test_case;

crate::test_case! {
    /// Return EPERM if the parent directory of the named file has its immutable flag set
    immutable_append_undeletable_file, root, FileSystemFeature::Chflags
}
fn immutable_append_undeletable_file(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_supported_and_error_flags(
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
        FileType::Regular,
        unlink,
        |path| !path.exists(),
    );

    // rmdir/09.t
    // TODO: Failure on ZFS with SF_APPEND
    assert_flags_named_file(ctx, &flags, &valid_flags, FileType::Dir, rmdir, |path| {
        !path.exists()
    });

    // rename/06.t
    // TODO: Failure on ZFS with SF_APPEND
    // TODO: Missing multiple file types
    assert_flags_named_file(
        ctx,
        &flags,
        &valid_flags,
        FileType::Regular,
        |from| {
            let to = ctx.gen_path();
            rename(from, &*to)
        },
        |from| !from.exists(),
    );
}

crate::test_case! {
    /// Return EPERM if the parent directory of the named file has its immutable flag set
    immutable_append_parent, root, FileSystemFeature::Chflags
}
fn immutable_append_parent(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_supported_and_error_flags(
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
    immutable_parent, root, FileSystemFeature::Chflags
}
fn immutable_parent(ctx: &mut TestContext) {
    let (flags, valid_flags) = get_supported_and_error_flags(
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
