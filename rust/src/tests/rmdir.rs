use std::{
    fs::metadata,
    path::{Path, PathBuf},
    process::Command,
};

use nix::errno::Errno;

use crate::{config::Config, context::TestContext, tests::assert_mtime_changed, utils::rmdir};

use super::{
    assert_ctime_changed,
    errors::efault::efault_path_test_case,
    errors::{eloop::eloop_comp_test_case, erofs::erofs_named_test_case},
    errors::{enametoolong::enametoolong_comp_test_case, enoent::enoent_named_file_test_case},
    errors::{enametoolong::enametoolong_path_test_case, enotdir::enotdir_comp_test_case},
};

crate::test_case! {
    /// rmdir remove directory
    remove_dir
}
fn remove_dir(ctx: &mut TestContext) {
    let dir = ctx.create(crate::context::FileType::Dir).unwrap();
    assert!(metadata(&dir).unwrap().is_dir());
    assert!(rmdir(&dir).is_ok());
    assert!(!dir.exists());
}

crate::test_case! {
    /// rmdir updates parent ctime and mtime on success
    changed_time_parent_success
}
fn changed_time_parent_success(ctx: &mut TestContext) {
    let dir = ctx.create(crate::context::FileType::Dir).unwrap();
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
        let from = ctx.create(crate::context::FileType::Dir)?;
        let path = ctx.create(crate::context::FileType::Dir)?;
        let mut mount = Command::new("mount");

        if cfg!(target_os = "linux") {
            mount.arg("--bind");
        } else {
            mount.args(["-t", "nullfs"]);
        }

        let result = mount.arg(&from).arg(&path).output()?;
        assert!(result.status.success());

        Ok(Self { path })
    }
}

impl Drop for DummyMnt {
    fn drop(&mut self) {
        let umount = Command::new("umount").arg(&self.path).output();
        if !std::thread::panicking() {
            assert!(matches!(umount, Ok(res) if res.status.success()));
        }
    }
}

#[cfg(target_os = "linux")]
fn has_mount_cap(_: &Config, _: &Path) -> anyhow::Result<()> {
    use caps::{has_cap, CapSet, Capability};

    if !has_cap(None, CapSet::Effective, Capability::CAP_SYS_ADMIN)? {
        anyhow::bail!("process doesn't have the CAP_SYS_ADMIN cap to mount the dummy file system")
    }

    Ok(())
}

#[cfg(target_os = "freebsd")]
fn has_mount_cap(_: &Config, _: &Path) -> anyhow::Result<()> {
    use nix::unistd::Uid;
    use sysctl::{Ctl, CtlValue, Sysctl};

    const MOUNT_CTL: &str = "vfs.usermount";

    let ctl = Ctl::new(MOUNT_CTL)?;

    if !Uid::effective().is_root() && ctl.value()? == CtlValue::Int(0) {
        anyhow::bail!("process doesn't have the rights to mount the dummy file system")
    }

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
fn has_mount_cap(_: &Config, _: &Path) -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        anyhow::bail!("process is not root, cannot mount dummy file system")
    }

    Ok(())
}

crate::test_case! {
    /// rmdir return EBUSY if the directory to be removed is the mount point for a mounted file system
    ebusy; has_mount_cap
}
fn ebusy(ctx: &mut TestContext) {
    let dummy_mount = DummyMnt::new(ctx).unwrap();
    assert_eq!(rmdir(&dummy_mount.path), Err(Errno::EBUSY));
}

// rmdir/02.t
enametoolong_comp_test_case!(rmdir);

// rmdir/03.t
enametoolong_path_test_case!(rmdir);

// rmdir/04.t
enoent_named_file_test_case!(rmdir);

// rmdir/05.t
eloop_comp_test_case!(rmdir);

crate::test_case! {
    /// rmdir returns EINVAL if the last component of the path is '.'
    // rmdir/12.t
    einval_dot
}
fn einval_dot(ctx: &mut TestContext) {
    assert_eq!(rmdir(&ctx.base_path().join(".")), Err(Errno::EINVAL));
}

// rmdir/14.t
erofs_named_test_case!(rmdir);

// rmdir/15.t
efault_path_test_case!(rmdir, nix::libc::rmdir);
