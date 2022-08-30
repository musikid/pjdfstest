use std::{
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    path::Path,
    process::Command,
};

use nix::errno::Errno;

use crate::{runner::context::SerializedTestContext, utils::rmdir};

/// Mount a dummy file system on a directory and execute the provided function with the directory's path.
fn mount_dir<F>(ctx: &mut SerializedTestContext, f: F)
where
    F: FnOnce(&Path),
{
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

    let res = catch_unwind(AssertUnwindSafe(|| f(&to)));

    let umount = Command::new("umount").arg(&to).output().unwrap();
    assert!(umount.status.success());

    if let Err(e) = res {
        resume_unwind(e);
    }
}

crate::test_case! {
    /// rmdir return EBUSY if the directory to be removed is the mount point for a mounted file system
    rmdir_mounted, serialized, root
}
fn rmdir_mounted(ctx: &mut SerializedTestContext) {
    mount_dir(ctx, |mntpoint| {
        assert_eq!(rmdir(mntpoint), Err(Errno::EBUSY))
    });
}
