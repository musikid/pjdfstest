use std::{fs::metadata, path::PathBuf, process::Command};

use nix::{errno::Errno, sys::stat::lstat};

use crate::{runner::context::TestContext, tests::assert_mtime_changed, utils::rmdir};

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

/// Dummy mountpoint to check that rmdir returns EBUSY when using it on a mountpoint.
struct DummyMnt {
    path: PathBuf,
}

impl DummyMnt {
    fn new(ctx: &mut TestContext) -> anyhow::Result<Self> {
        // We don't really care about a specific type of file system here, the directory just have to be a mount point
        let from = ctx.create(crate::runner::context::FileType::Dir)?;
        let path = ctx.create(crate::runner::context::FileType::Dir)?;
        let mut mount = Command::new("mount");

        if cfg!(target_os = "linux") {
            mount.arg("--bind");
        } else {
            mount.args(["-t", "nullfs"]);
        }

        let result = mount.arg(&from).arg(&path).output()?;
        debug_assert!(result.status.success());

        Ok(Self { path })
    }
}

impl Drop for DummyMnt {
    fn drop(&mut self) {
        let umount = Command::new("umount").arg(&self.path).output();
        debug_assert!(matches!(umount, Ok(res) if res.status.success()));
    }
}

crate::test_case! {
    /// rmdir return EBUSY if the directory to be removed is the mount point for a mounted file system
    ebusy, root
}
fn ebusy(ctx: &mut TestContext) {
    let dummy_mount = DummyMnt::new(ctx).unwrap();
    assert_eq!(rmdir(&dummy_mount.path), Err(Errno::EBUSY));
}
