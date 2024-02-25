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
    context::{FileType, SerializedTestContext, TestContext},
    test::{FileFlags, FileSystemFeature},
};

use super::{
    assert_ctime_changed, assert_ctime_unchanged,
    errors::efault::efault_path_test_case,
    errors::eloop::eloop_comp_test_case,
    errors::enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case},
    errors::enoent::{enoent_comp_test_case, enoent_named_file_test_case},
    errors::enotdir::enotdir_comp_test_case,
    errors::erofs::erofs_named_test_case,
};

//TODO: Split tests with unprivileged tests for user flags

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
    set_flags, root, FileSystemFeature::Chflags => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn set_flags(ctx: &mut TestContext, ft: FileType) {
    let (flags, user_flags, system_flags) = get_flags(ctx);

    let file = ctx.create(ft.clone()).unwrap();
    assert!(chflags(&file, FileFlag::empty()).is_ok());

    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(chflags(&file, FileFlag::empty()).is_ok());
        assert!(chflags(&file, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        assert_eq!(file_flags, flags_set.bits() as fflags_t);
    }

    // Check with lchflags

    let file = ctx.create(ft).unwrap();
    assert!(chflags(&file, FileFlag::empty()).is_ok());

    #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(lchflags(&file, FileFlag::empty()).is_ok());
        assert!(lchflags(&file, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        assert_eq!(file_flags, flags_set.bits() as fflags_t);
    }
}

crate::test_case! {
    /// chflags changes flags while following symlinks
    set_flags_symlink, root, FileSystemFeature::Chflags
}
fn set_flags_symlink(ctx: &mut TestContext) {
    let (flags, user_flags, system_flags) = get_flags(ctx);

    let file = ctx.create(FileType::Regular).unwrap();
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let original_link_flags = lstat(&link).unwrap().st_flags;

    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(chflags(&link, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        let link_flags = lstat(&link).unwrap().st_flags;
        assert_eq!(file_flags, flags_set.bits() as fflags_t);
        assert_eq!(link_flags, original_link_flags);
        assert!(chflags(&link, FileFlag::empty()).is_ok());
    }
}

#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
crate::test_case! {
    /// lchflags changes flags without following symlinks
    lchflags_set_flags_no_follow_symlink, root, FileSystemFeature::Chflags
}
#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
fn lchflags_set_flags_no_follow_symlink(ctx: &mut TestContext) {
    let (flags, user_flags, system_flags) = get_flags(ctx);

    let file = ctx.create(FileType::Regular).unwrap();
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let original_file_flags = stat(&file).unwrap().st_flags;

    for flags_set in [flags, user_flags, system_flags, FileFlag::empty()] {
        assert!(lchflags(&link, flags_set).is_ok());
        let file_flags = stat(&file).unwrap().st_flags;
        let link_flags = lstat(&link).unwrap().st_flags;
        assert_eq!(file_flags, original_file_flags);
        assert_eq!(link_flags, flags_set.bits() as fflags_t);
        assert!(lchflags(&link, FileFlag::empty()).is_ok());
    }
}

crate::test_case! {
    // successful chflags(2) updates ctime
    changed_ctime_success, root => [Regular, Dir, Fifo, Block, Char, Socket]
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

// chflags/01.t
enotdir_comp_test_case!(chflags(~path, FileFlag::empty()));

// chflags/02.t
enametoolong_comp_test_case!(chflags(~path, FileFlag::empty()));
enametoolong_path_test_case!(chflags(~path, FileFlag::empty()));

// chflags/04.t
enoent_named_file_test_case!(chflags(~path, FileFlag::empty()));

// chflags/04.t
enoent_comp_test_case!(chflags(~path, FileFlag::empty()));

// chflags/06.t
eloop_comp_test_case!(chflags(~path, FileFlag::empty()));

// chflags/09.t
#[cfg(target_os = "freebsd")]
crate::test_case! {
    /// chflags returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK is set and
    /// securelevel is greater than 0
    securelevel, root, FileSystemFeature::Chflags =>
        [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
#[cfg(target_os = "freebsd")]
fn securelevel(ctx: &mut TestContext, ft: FileType) {
    use jail::process::Jailed;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let jail = jail::StoppedJail::new("/")
        .name("pjdfstest_chflags_securelevel")
        .param("allow.chflags", jail::param::Value::Int(1))
        .param("securelevel", jail::param::Value::Int(1));
    let jail = jail.start().unwrap();
    ctx.set_jail(jail);

    for flag in [FileFlags::SF_IMMUTABLE, FileFlags::SF_APPEND, FileFlags::SF_NOUNLINK] {
        let file = ctx.create(ft.clone()).unwrap();
        lchflags(&file, flag.into()).unwrap();

        // Since this is a multithreaded application, we can't simply fork and chflags().  Instead,
        // execute a child process to test the operation.
        let r = std::process::Command::new("/bin/chflags")
            .args(["-h", "0"])
            .arg(ctx.base_path().join(&file))
            .jail(&jail)
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert!(OsStr::from_bytes(&r.stderr).to_string_lossy().contains("Operation not permitted"));
    }
}

// chflags/13.t
efault_path_test_case!(chflags, |ptr| nix::libc::chflags(ptr, 0));

// chflags/12.t
erofs_named_test_case!(chflags(~path, FileFlag::empty()));
