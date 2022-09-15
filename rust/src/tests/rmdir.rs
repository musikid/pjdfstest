use std::{
    fs::metadata,
    panic::{catch_unwind, resume_unwind},
    process::Command,
};

use nix::{errno::Errno, sys::stat::lstat};

use crate::{
    runner::context::{SerializedTestContext, TestContext},
    tests::assert_mtime_changed,
    utils::rmdir,
};

use super::{assert_ctime_changed, errors::enotdir::enotdir_comp_test_case};

crate::test_case! {
    /// rmdir remove directory
    remove_dir
}
fn remove_dir(ctx: &mut TestContext) {
    let dir = ctx.create(crate::runner::context::FileType::Dir).unwrap();
    assert!(metadata(&dir).unwrap().is_dir());
    assert!(rmdir(&dir).is_ok());
    assert!(!dir.exists());
}

crate::test_case! {
    /// rmdir updates parent ctime and mtime on success
    changed_time_parent_success
}
fn changed_time_parent_success(ctx: &mut TestContext) {
    let dir = ctx.create(crate::runner::context::FileType::Dir).unwrap();
    assert_ctime_changed(ctx, ctx.base_path(), || {
        assert_mtime_changed(ctx, ctx.base_path(), || {
            assert!(rmdir(&dir).is_ok());
        });
    });
}

// rmdir/01.t
enotdir_comp_test_case!(rmdir);

crate::test_case! {
    /// rmdir return EBUSY if the directory to be removed is the mount point for a mounted file system
    ebusy, serialized, root
}
fn ebusy(ctx: &mut SerializedTestContext) {
    // We don't really care about a specific type of file system here, the directory just have to be a mount point
    let from = ctx.create(crate::runner::context::FileType::Dir).unwrap();
    let to = ctx.create(crate::runner::context::FileType::Dir).unwrap();
    let mut mount = Command::new("mount");

    if cfg!(target_os = "linux") {
        mount.arg("--bind");
    } else {
        mount.args(["-t", "nullfs"]);
    }

    let result = mount.arg(&from).arg(&to).output().unwrap();
    assert!(result.status.success());

    let res = catch_unwind(|| assert_eq!(rmdir(&to), Err(Errno::EBUSY)));

    let umount = Command::new("umount").arg(&to).output().unwrap();
    assert!(umount.status.success());

    if let Err(e) = res {
        resume_unwind(e);
    }
}
