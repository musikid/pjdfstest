use std::{collections::HashSet, iter::once};

use nix::{
    errno::Errno,
    libc::fflags_t,
    sys::stat::{lstat, stat, FileFlag},
    unistd::chflags,
};
use once_cell::sync::Lazy;

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
use crate::utils::lchflags;
use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    test::{FileFlags, FileSystemFeature},
};

use super::{assert_ctime_changed, assert_ctime_unchanged};

//TODO: Does the split user/system flags make sense? Besides, only FreeBSD ones are specified in the original test suite.

const USER_FLAGS: Lazy<HashSet<FileFlags>> = Lazy::new(|| {
    HashSet::from([
        FileFlags::UF_NODUMP,
        FileFlags::UF_IMMUTABLE,
        FileFlags::UF_APPEND,
        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        FileFlags::UF_NOUNLINK,
        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        FileFlags::UF_OPAQUE,
    ])
});

const SYSTEM_FLAGS: Lazy<HashSet<FileFlags>> = Lazy::new(|| {
    HashSet::from([
        FileFlags::SF_ARCHIVED,
        FileFlags::SF_IMMUTABLE,
        FileFlags::SF_APPEND,
        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        FileFlags::SF_NOUNLINK,
    ])
});

fn get_flags(ctx: &TestContext) -> (FileFlag, FileFlag, FileFlag) {
    let allflags: FileFlag = ctx
        .features_config()
        .file_flags
        .iter()
        .copied()
        .map(Into::into)
        .collect();

    let user_flags: FileFlag = ctx
        .features_config()
        .file_flags
        .intersection(&USER_FLAGS)
        .copied()
        .map(Into::into)
        .collect();

    let system_flags: FileFlag = ctx
        .features_config()
        .file_flags
        .intersection(&SYSTEM_FLAGS)
        .copied()
        .map(Into::into)
        .collect();

    (allflags, user_flags, system_flags)
}

crate::test_case! {
    /// chflags(2) set the flags provided for the file.
    set_flags, FileSystemFeature::Chflags => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn set_flags(ctx: &mut TestContext, ft: FileType) {
    let (flags, user_flags, system_flags) = get_flags(ctx);

    let file = ctx.create(ft.clone()).unwrap();
    let file_flags = stat(&file).unwrap().st_flags;
    assert_eq!(file_flags, FileFlag::empty().bits() as fflags_t);

    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(chflags(&file, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        assert_eq!(file_flags, flags_set.bits() as fflags_t);
    }

    // Check with lchflags

    let file = ctx.create(ft).unwrap();
    let file_flags = stat(&file).unwrap().st_flags;
    assert_eq!(file_flags, FileFlag::empty().bits() as fflags_t);

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(lchflags(&file, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        assert_eq!(file_flags, flags_set.bits() as fflags_t);
    }
}

crate::test_case! {
    /// chflags changes flags while following symlinks
    set_flags_symlink, FileSystemFeature::Chflags
}
fn set_flags_symlink(ctx: &mut TestContext) {
    let (flags, user_flags, system_flags) = get_flags(ctx);

    let file = ctx.create(FileType::Regular).unwrap();
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let file_flags = stat(&file).unwrap().st_flags;
    let link_flags = lstat(&link).unwrap().st_flags;
    assert_eq!(file_flags, FileFlag::empty().bits() as fflags_t);
    assert_eq!(link_flags, FileFlag::empty().bits() as fflags_t);

    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(chflags(&file, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        let link_flags = lstat(&link).unwrap().st_flags;
        assert_eq!(file_flags, flags_set.bits() as fflags_t);
        assert_eq!(link_flags, FileFlag::empty().bits() as fflags_t);
    }
}

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
crate::test_case! {
    /// lchflags changes flags without following symlinks
    lchflags_set_flags_no_follow_symlink, FileSystemFeature::Chflags
}
#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
fn lchflags_set_flags_no_follow_symlink(ctx: &mut TestContext) {
    let (flags, user_flags, system_flags) = get_flags(ctx);

    let file = ctx.create(FileType::Regular).unwrap();
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let file_flags = stat(&file).unwrap().st_flags;
    let link_flags = lstat(&link).unwrap().st_flags;
    assert_eq!(file_flags, FileFlag::empty().bits() as fflags_t);
    assert_eq!(link_flags, FileFlag::empty().bits() as fflags_t);

    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(lchflags(&file, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        let link_flags = lstat(&link).unwrap().st_flags;
        assert_eq!(file_flags, FileFlag::empty().bits() as fflags_t);
        assert_eq!(link_flags, flags_set.bits() as fflags_t);
    }
}

crate::test_case! {
    // successful chflags(2) updates ctime
    changed_ctime_success => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn changed_ctime_success(ctx: &mut TestContext, ft: FileType) {
    let allflags: Vec<FileFlag> = ctx
        .features_config()
        .file_flags
        .iter()
        .cloned()
        .map(Into::into)
        .collect();

    let file = ctx.create(ft.clone()).unwrap();

    for flag in allflags.iter().chain(once(&FileFlag::empty())) {
        assert_ctime_changed(ctx, &file, || {
            assert!(chflags(&file, *flag).is_ok());
        });
    }

    let file = ctx.create(ft).unwrap();

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    for flag in allflags.into_iter().chain(once(FileFlag::empty())) {
        assert_ctime_changed(ctx, &file, || {
            assert!(lchflags(&file, flag).is_ok());
        });
    }
}
crate::test_case! {
    // unsuccessful chflags(2) does not update ctime
    unchanged_ctime_failed, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn unchanged_ctime_failed(ctx: &mut SerializedTestContext, ft: FileType) {
    let allflags: Vec<FileFlag> = ctx
        .features_config()
        .file_flags
        .iter()
        .cloned()
        .map(Into::into)
        .collect();

    let user = ctx.get_new_user();

    let file = ctx.create(ft.clone()).unwrap();

    for flag in allflags.iter().chain(once(&FileFlag::empty())) {
        assert_ctime_unchanged(ctx, &file, || {
            ctx.as_user(&user, None, || {
                assert_eq!(chflags(&file, *flag), Err(Errno::EPERM));
            })
        });
    }

    let file = ctx.create(ft).unwrap();

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    for flag in allflags.into_iter().chain(once(FileFlag::empty())) {
        assert_ctime_unchanged(ctx, &file, || {
            ctx.as_user(&user, None, || {
                assert_eq!(lchflags(&file, flag), Err(Errno::EPERM));
            })
        });
    }
}
