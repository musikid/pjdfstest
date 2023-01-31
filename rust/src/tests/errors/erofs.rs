use std::{
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    path::Path,
    process::Command,
};

use crate::utils::get_mountpoint;

enum RemountOptions {
    ReadOnly,
    ReadWrite,
}

/// Guard to allow execution of this test only if it's allowed to run.
pub(crate) fn can_run_erofs(conf: &crate::config::Config, _: &Path) -> anyhow::Result<()> {
    if !conf.settings.allow_erofs {
        anyhow::bail!("EROFS is not enabled in the configuration file")
    }

    Ok(())
}

/// Remount the file system mounted at `mountpoint` with the provided options.
fn remount<P: AsRef<Path>>(mountpoint: P, options: RemountOptions) -> Result<(), anyhow::Error> {
    if mountpoint.as_ref().parent().is_none() {
        anyhow::bail!("Cannot remount root file system")
    }

    let opt = match options {
        RemountOptions::ReadOnly => "ro",
        RemountOptions::ReadWrite => "rw",
    };

    #[cfg(target_os = "linux")]
    let opt = format!("remount,{opt}");

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
        anyhow::bail!(
            "Failed to remount: {}",
            String::from_utf8_lossy(&mount_result.stderr)
        )
    }

    Ok(())
}

/// Execute a function with a read-only file system and restore read/write flags after.
// TODO: Get the original flags
pub(crate) fn with_readonly_fs<F, P: AsRef<Path>>(path: P, f: F)
where
    F: FnOnce(),
{
    let mountpoint = get_mountpoint(path.as_ref()).unwrap();

    remount(mountpoint, RemountOptions::ReadOnly).unwrap();

    let res = catch_unwind(AssertUnwindSafe(f));

    remount(mountpoint, RemountOptions::ReadWrite).unwrap();

    if let Err(e) = res {
        resume_unwind(e);
    }
}

/// Create a test case which asserts that the syscall returns EROFS
/// if the path resides on a read-only file system.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// erofs_new_file_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// erofs_new_file_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```
/// erofs_new_file_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ```
macro_rules! erofs_new_file_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EROFS if the path for the file to be created resides on a read-only file system")]
            erofs_new_file, serialized, root; crate::tests::errors::erofs::can_run_erofs
        }
        fn erofs_new_file(ctx: &mut crate::context::SerializedTestContext) {
            use crate::tests::errors::erofs::with_readonly_fs;
            let path = ctx.base_path().to_owned();
            let file = ctx.gen_path();
            with_readonly_fs(path, || {
                $( assert_eq!($f(ctx, &file), Err(nix::errno::Errno::EROFS)); )+
            });
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        crate::tests::errors::erofs::erofs_new_file_test_case!($syscall, |_ctx, path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use erofs_new_file_test_case;

/// Create a test case which asserts that the syscall returns EROFS
/// if the named file resides on a read-only file system.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// erofs_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// erofs_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```
/// erofs_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ```
macro_rules! erofs_named_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns EROFS if the named file resides on a read-only file system")]
            erofs_named, serialized, root; crate::tests::errors::erofs::can_run_erofs
        }
        fn erofs_named(ctx: &mut crate::context::SerializedTestContext) {
            use crate::tests::errors::erofs::with_readonly_fs;
            use crate::context::FileType;
            let path = ctx.base_path().to_owned();
            let file = ctx.new_file(FileType::Regular).name(path.join("file")).create().unwrap();
            with_readonly_fs(path, || {
                $( assert_eq!($f(ctx, &file), Err(nix::errno::Errno::EROFS)); )+
            });
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        crate::tests::errors::erofs::erofs_named_test_case!($syscall, |_ctx, path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use erofs_named_test_case;
