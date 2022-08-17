use std::{
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    path::Path,
    process::Command,
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    utils::{chmod, lchmod, lchown, link, rename, rmdir, symlink},
};
use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{lstat, Mode},
    unistd::{chown, mkdir, mkfifo, truncate, unlink},
};
#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
use nix::{sys::stat::FileFlag, unistd::chflags};

fn get_mountpoint(base_path: &Path) -> Result<&Path, anyhow::Error> {
    let base_dev = lstat(base_path)?.st_dev;

    let mut mountpoint = base_path;
    loop {
        let current = match mountpoint.parent() {
            Some(p) => p,
            // Root
            _ => return Ok(mountpoint),
        };
        let current_dev = lstat(current)?.st_dev;

        if current_dev != base_dev {
            break;
        }

        mountpoint = current;
    }

    Ok(mountpoint)
}

enum RemountOptions {
    ReadOnly,
    ReadWrite,
}

fn remount<P: AsRef<Path>>(mountpoint: P, options: RemountOptions) -> Result<(), anyhow::Error> {
    if mountpoint.as_ref().parent().is_none() {
        anyhow::bail!("Cannot remount root file system")
    }

    let opt = match options {
        RemountOptions::ReadOnly => "ro",
        RemountOptions::ReadWrite => "rw",
    };

    #[cfg(any(target_os = "linux"))]
    let opt = format!("remount,{opt}");

    #[cfg(not(any(target_os = "linux")))]
    let opt = String::from(opt);

    let mut cmd = Command::new("mount");

    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    cmd.arg("-u");

    // XXX: It is possible to do it directly with mount(2) but we use the CLI for now
    let mount_result = cmd.arg("-o").arg(opt).arg(mountpoint.as_ref()).output()?;

    if !mount_result.status.success() {
        let error = String::from_utf8_lossy(&mount_result.stderr);
        anyhow::bail!("Failed to remount: {error}")
    }

    Ok(())
}

/// Execute a function with a read-only file system and restore read/write flags after.
// TODO: Get the original flags
fn with_readonly_fs<F>(path: &Path, f: F)
where
    F: FnOnce(),
{
    let mountpoint = get_mountpoint(path).unwrap();

    remount(mountpoint, RemountOptions::ReadOnly).unwrap();

    let res = catch_unwind(AssertUnwindSafe(f));

    remount(mountpoint, RemountOptions::ReadWrite).unwrap();

    if let Err(e) = res {
        resume_unwind(e);
    }
}

crate::test_case! {
    /// Return EROFS if the named file resides on a read-only file system
    // TODO: Assert that it works in a write context? We already guarentee it with the other tests?
    read_only, serialized, root
}
fn read_only(ctx: &mut SerializedTestContext) {
    let path = ctx.base_path().to_owned();
    let fake_file = ctx.create(FileType::Regular).unwrap();
    let fake_path = ctx.gen_path();
    with_readonly_fs(&path, || {
        #[cfg(any(
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "macos",
            target_os = "ios",
            target_os = "watchos",
        ))]
        // chflags/12.t
        assert_eq!(chflags(&fake_file, FileFlag::empty()), Err(Errno::EROFS));

        // chmod/09.t
        assert_eq!(chmod(&fake_file, Mode::empty()), Err(Errno::EROFS));
        #[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
        assert_eq!(lchmod(&fake_file, Mode::empty()), Err(Errno::EROFS));

        // chown/09.t
        let user = ctx.get_new_user();
        assert_eq!(
            chown(&fake_file, Some(user.uid), Some(user.gid)),
            Err(Errno::EROFS)
        );
        assert_eq!(
            lchown(&fake_file, Some(user.uid), Some(user.gid)),
            Err(Errno::EROFS)
        );

        // (f)truncate/12.t
        assert_eq!(truncate(&fake_file, 0), Err(Errno::EROFS));

        // link/16.t
        assert_eq!(link(&fake_file, &fake_path), Err(Errno::EROFS));

        // mkdir/09.t
        assert_eq!(mkdir(&fake_path, Mode::empty()), Err(Errno::EROFS));

        // mkfifo/08.t
        assert_eq!(mkfifo(&fake_path, Mode::empty()), Err(Errno::EROFS));

        // open/14.t
        assert_eq!(
            open(&fake_file, OFlag::O_WRONLY, Mode::empty()),
            Err(Errno::EROFS)
        );
        assert_eq!(
            open(&fake_file, OFlag::O_RDWR, Mode::empty()),
            Err(Errno::EROFS)
        );
        assert_eq!(
            open(&fake_file, OFlag::O_RDONLY | OFlag::O_TRUNC, Mode::empty()),
            Err(Errno::EROFS)
        );
        // open/15.t
        assert_eq!(
            open(&fake_path, OFlag::O_RDONLY | OFlag::O_CREAT, Mode::empty()),
            Err(Errno::EROFS)
        );

        // rename/16.t
        assert_eq!(rename(&fake_file, &fake_path), Err(Errno::EROFS));

        // rmdir/14.t
        assert_eq!(rmdir(&fake_file), Err(Errno::EROFS));

        // rename/16.t
        assert_eq!(symlink(&fake_file, &fake_path), Err(Errno::EROFS));

        // unlink/12.t
        assert_eq!(unlink(&fake_file), Err(Errno::EROFS));
    });
}
