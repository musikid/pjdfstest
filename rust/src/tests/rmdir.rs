use std::fs::metadata;

use nix::{errno::Errno, sys::stat::lstat};

use crate::{runner::context::TestContext, tests::assert_mtime_changed, utils::rmdir};

use super::{assert_ctime_changed, errors::enotdir::assert_enotdir_comp};

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
assert_enotdir_comp!(rmdir);
