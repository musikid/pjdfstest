use std::{
    collections::HashSet,
    fmt::Debug,
    fs::{metadata, symlink_metadata},
    os::freebsd::fs::MetadataExt as _,
    path::Path,
};

use nix::{errno::Errno, sys::stat::FileFlag, unistd::chflags};

use crate::{
    context::{FileType, TestContext},
    flags::FileFlags,
    utils::lchflags,
};

/// Return flags which intersects with the provided ones
/// and those available in the configuration,
/// along with the other available in the configuration (representing the flags which don't trigger errors in this context).
pub fn get_supported_error_flags_and_valid_flags(
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
    valid_flags: &[FileFlags],
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
    let chflags_variant = if matches!(created_type, Some(FileType::Symlink(..))) {
        lchflags
    } else {
        chflags
    };
    let metadata_variant = if matches!(created_type, Some(FileType::Symlink(..))) {
        symlink_metadata
    } else {
        metadata
    };

    for &flag in flags {
        let raw_flag: FileFlag = flag.into();
        let (flagged_file, file) = get_files();

        chflags_variant(&flagged_file, raw_flag).unwrap();

        // TODO: Add flag names list from FileFlag init when nix will be upgraded
        // BUG: Using a reference to the path doesn't work as expected, "does not live long enough"
        let set_flags = metadata_variant(flagged_file.clone()).unwrap().st_flags();
        assert!(
            set_flags as u64 & raw_flag.bits() > 0,
            "File should have {flag} set but only have {set_flags}"
        );

        assert!(
            matches!(f(&file), Err(Errno::EPERM)),
            "{flag} does not trigger EPERM"
        );
        assert!(!check(&file), "Error file check failed for {flag}");

        chflags_variant(&flagged_file, FileFlag::empty()).unwrap();

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

/// Helper to generate `immutable_append_named` test cases,
/// which asserts that the syscall returns EPERM
/// if the named file has its immutable or append-only flag set.
pub(crate) fn immutable_append_named_helper<T: Debug, F, C>(ctx: &TestContext, f: F, check: C)
where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let (flags, valid_flags) = get_supported_error_flags_and_valid_flags(
        &ctx.features_config().file_flags,
        &[FileFlags::IMMUTABLE_FLAGS, FileFlags::APPEND_ONLY_FLAGS].concat(),
    );

    assert_flags_named_file(ctx, &flags, &valid_flags, FileType::Regular, f, check)
}

/// Create a test case which asserts that the syscall returns EPERM
/// if the named file has its immutable or append-only flag set.
/// Takes as arguments the syscall name,
/// the function to produce a result which can be checked against and the check
/// function which asserts that it worked using the previously produced result.
macro_rules! immutable_append_named_test_case {
    ($syscall: ident, $f: expr, $c: expr) => {
        $crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EPERM if the named file has its immutable or append-only flag set")]
            immutable_append_named, root, $crate::test::FileSystemFeature::Chflags;
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::IMMUTABLE_FLAGS),
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::APPEND_ONLY_FLAGS)
        }
        fn immutable_append_named(ctx: &mut $crate::context::TestContext) {
            $crate::tests::errors::eperm::flag::immutable_append_named_helper(ctx, $f, $c)
        }
    };
}
pub(crate) use immutable_append_named_test_case;

/// Helper to generate `immutable_append_named` test cases,
/// which asserts that the syscall returns EPERM
/// if the named file has its immutable, undeletable or append-only flag set.
pub(crate) fn immutable_append_undeletable_named_helper<T: Debug, F, C>(
    ctx: &TestContext,
    f: F,
    check: C,
    ft: FileType,
) where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let (flags, valid_flags) = get_supported_error_flags_and_valid_flags(
        &ctx.features_config().file_flags,
        &[
            FileFlags::IMMUTABLE_FLAGS,
            FileFlags::APPEND_ONLY_FLAGS,
            FileFlags::UNDELETABLE_FLAGS,
        ]
        .concat(),
    );

    assert_flags_named_file(ctx, &flags, &valid_flags, ft, f, check)
}

/// Create a test case which asserts that the syscall returns EPERM
/// if the named file has its immutable, undeletable or append-only flag set.
/// Takes as arguments the syscall name,
/// the function to produce a result which can be checked against, the check
/// function which asserts that it worked using the previously produced result,
/// and optionally the FileType to operate on.
macro_rules! immutable_append_undeletable_named_test_case {
    ($syscall: ident, $f: expr, $c: expr, $ft: expr) => {
        $crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EPERM if the named file has its immutable, undeletable or append-only flag set")]
            immutable_append_undeletable_named, root, $crate::test::FileSystemFeature::Chflags;
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::IMMUTABLE_FLAGS),
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::APPEND_ONLY_FLAGS),
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::UNDELETABLE_FLAGS)
        }
        fn immutable_append_undeletable_named(ctx: &mut $crate::context::TestContext) {
            $crate::tests::errors::eperm::flag::immutable_append_undeletable_named_helper(
                ctx, $f, $c, $ft,
            )
        }
    };
    ($syscall: ident, $f: expr, $c: expr) => {
        $crate::tests::errors::eperm::flag::immutable_append_undeletable_named_test_case!(
            $syscall,
            $f,
            $c,
            $crate::context::FileType::Regular
        );
    };
}
pub(crate) use immutable_append_undeletable_named_test_case;

/// Helper to generate `immutable_append_parent` test cases,
/// which asserts that the syscall returns EPERM
/// if the parent directory of the named file has its immutable or append-only flag set.
pub(crate) fn immutable_append_parent_helper<T: Debug, F, C>(
    ctx: &TestContext,
    f: F,
    check: C,
    ft: FileType,
) where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let (flags, valid_flags) = get_supported_error_flags_and_valid_flags(
        &ctx.features_config().file_flags,
        &[FileFlags::IMMUTABLE_FLAGS, FileFlags::APPEND_ONLY_FLAGS].concat(),
    );

    assert_flags_parent(ctx, &flags, &valid_flags, Some(ft), f, check)
}

/// Create a test case which asserts that the syscall returns EPERM
/// if the parent directory of the named file has its immutable or append-only flag set.
/// Takes as arguments the syscall name,
/// the function to produce a result which can be checked against, the check
/// function which asserts that it worked using the previously produced result,
/// and optionally the FileType to operate on.
macro_rules! immutable_append_parent_test_case {
    ($syscall: ident, $f: expr, $c: expr, $ft: expr) => {
        $crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EPERM if the parent directory of the named file has its immutable or append-only flag set")]
            immutable_append_parent, root, $crate::test::FileSystemFeature::Chflags;
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::IMMUTABLE_FLAGS),
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::APPEND_ONLY_FLAGS)
        }
        fn immutable_append_parent(ctx: &mut $crate::context::TestContext) {
            $crate::tests::errors::eperm::flag::immutable_append_parent_helper(
                ctx, $f, $c, $ft,
            )
        }
    };
    ($syscall: ident, $f: expr, $c: expr) => {
        $crate::tests::errors::eperm::flag::immutable_append_parent_test_case!(
            $syscall,
            $f,
            $c,
            $crate::context::FileType::Regular
        );
    };
}
pub(crate) use immutable_append_parent_test_case;

/// Helper to generate `immutable_parent` test cases,
/// which asserts that the syscall returns EPERM
/// if the parent directory of the to be created file has its immutable flag set.
pub(crate) fn immutable_parent_helper<T: Debug, F, C>(ctx: &TestContext, f: F, check: C)
where
    F: Fn(&Path) -> nix::Result<T>,
    C: Fn(&Path) -> bool,
{
    let (flags, valid_flags) = get_supported_error_flags_and_valid_flags(
        &ctx.features_config().file_flags,
        FileFlags::IMMUTABLE_FLAGS,
    );

    assert_flags_parent(ctx, &flags, &valid_flags, None, f, check)
}

/// Create a test case which asserts that the syscall returns EPERM
/// if the parent directory of the to be created file has its immutable flag set.
/// Takes as arguments the syscall name,
/// the function to produce a result which can be checked against, the check
/// function which asserts that it worked using the previously produced result.
macro_rules! immutable_parent_test_case {
    ($syscall: ident, $f: expr, $c: expr) => {
        $crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EPERM if the parent directory of the named file has its immutable or append-only flag set")]
            immutable_parent, root, $crate::test::FileSystemFeature::Chflags;
            $crate::tests::supports_any_flag!($crate::flags::FileFlags::IMMUTABLE_FLAGS)
        }
        fn immutable_parent(ctx: &mut $crate::context::TestContext) {
            $crate::tests::errors::eperm::flag::immutable_parent_helper(ctx, $f, $c)
        }
    };
}
pub(crate) use immutable_parent_test_case;
