use std::{
    ffi::OsStr,
    fs::metadata,
    os::unix::ffi::OsStrExt,
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
        let stderr = OsStr::from_bytes(&result.stderr).to_string_lossy();
        assert!(result.status.success(), "{}", stderr);

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
    if !Uid::effective().is_root()
        && !OsStr::from_bytes(&Command::new("lsvfs").output().unwrap().stdout)
            .to_string_lossy()
            .contains("nullfs")
    {
        anyhow::bail!("nullfs module is not loaded")
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

// rmdir/02.t
enametoolong_comp_test_case!(rmdir);

// rmdir/03.t
enametoolong_path_test_case!(rmdir);

// rmdir/04.t
enoent_named_file_test_case!(rmdir);

// rmdir/05.t
eloop_comp_test_case!(rmdir);

crate::test_case! {
    /// rmdir returns EEXIST or ENOTEMPTY if the named directory
    /// contains files other than '.' and '..' in it
    // rmdir/06.t
    eexist_enotempty_non_empty_dir => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn eexist_enotempty_non_empty_dir(ctx: &mut TestContext, ft: crate::context::FileType) {
    ctx.new_file(ft)
        .name(ctx.base_path().join("file"))
        .create()
        .unwrap();

    assert!(matches!(
        rmdir(ctx.base_path()),
        Err(Errno::EEXIST | Errno::ENOTEMPTY)
    ));
}

crate::test_case! {
    /// rmdir returns EINVAL if the last component of the path is '.'
    // rmdir/12.t
    einval_dot
}
fn einval_dot(ctx: &mut TestContext) {
    assert_eq!(rmdir(&ctx.base_path().join(".")), Err(Errno::EINVAL));
}

crate::test_case! {
    /// rmdir returns EEXIST or ENOTEMPTY if the last component of the path is '..'
    // rmdir/12.t
    eexist_enotempty_dotdot
}
#[cfg_attr(target_os = "freebsd", allow(unused_variables))]
fn eexist_enotempty_dotdot(ctx: &mut TestContext) {
    // TODO: Not conforming to POSIX on FreeBSD
    // According to POSIX: EEXIST or ENOTEMPTY:
    // The path argument names a directory that is
    // not an empty directory,
    // or there are hard links to the directory other than dot or a single entry in dot-dot.
    #[cfg(not(target_os = "freebsd"))]
    {
        assert!(matches!(
            rmdir(&ctx.base_path().join("..")),
            Err(Errno::ENOTEMPTY | Errno::EEXIST)
        ));
    }
}

crate::test_case! {
    /// rmdir return EBUSY if the directory to be removed is the mount point for a mounted file system
    // rmdir/13.t
    ebusy; has_mount_cap
}
fn ebusy(ctx: &mut TestContext) {
    let dummy_mount = DummyMnt::new(ctx).unwrap();
    assert_eq!(rmdir(&dummy_mount.path), Err(Errno::EBUSY));
}

// rmdir/14.t
erofs_named_test_case!(rmdir);

// rmdir/15.t
efault_path_test_case!(rmdir, nix::libc::rmdir);
