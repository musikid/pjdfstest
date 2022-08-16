use nix::libc::{
    chmod, chown, link, mkdir, mkfifo, mknod, open, rename, rmdir, symlink, truncate, unlink,
};

use crate::runner::context::TestContext;

crate::test_case! {
    /// Return EFAULT if the path argument points outside the process's allocated address space
    path
}
fn path(_: &mut TestContext) {
    #![allow(unused_unsafe)]
    // TODO: Remove extra unsafe blocks

    /// Asserts that it returns EFAULT if the path argument points outside the process's allocated address space
    macro_rules! assert_ptr_invalid {
        (|$ptr: ident| $fn: expr) => {
            let f = |$ptr: *const _| unsafe { $fn };
            assert_ptr_invalid!(f);
        };

        ($fn: ident) => {
            let null_ptr = std::ptr::null();
            let invalid_ptr = std::ptr::NonNull::dangling();
            let invalid_ptr = invalid_ptr.as_ptr();

            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(null_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(invalid_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
        };
    }

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    // chflags/13.t
    assert_ptr_invalid!(|ptr| chflags(ptr, 0));
    // chmod/10.t
    assert_ptr_invalid!(|ptr| chmod(ptr, 0));
    // chown/10.t
    assert_ptr_invalid!(|ptr| chown(ptr, 0, 0));
    // mkdir/12.t
    assert_ptr_invalid!(|ptr| mkdir(ptr, 0o755));
    // mkfifo/12.t
    assert_ptr_invalid!(|ptr| mkfifo(ptr, 0o644));
    // mknod/10.t
    assert_ptr_invalid!(|ptr| mknod(ptr, 0o644, 0));
    // open/21.t
    assert_ptr_invalid!(|ptr| open(ptr, nix::libc::O_RDONLY));
    // rmdir/15.t
    assert_ptr_invalid!(rmdir);
    // (f)truncate/14.t
    assert_ptr_invalid!(|ptr| truncate(ptr, 0));
    // unlink/13.t
    assert_ptr_invalid!(unlink);
}

crate::test_case! {
    /// Return EFAULT if one of the pathnames specified is outside the process's allocated address space
    either_path
}
fn either_path(ctx: &mut TestContext) {
    use std::os::unix::ffi::OsStrExt;
    /// Asserts that it returns EFAULT if the path argument points outside the process's allocated address space
    macro_rules! assert_ptr_invalid {
        ($fn: ident) => {
            let file = ctx
                .create(crate::runner::context::FileType::Regular)
                .unwrap();
            let path = file.as_os_str().as_bytes();
            let path = std::ffi::CString::new(path).unwrap();
            let ptr = path.as_ptr();

            let null_ptr = std::ptr::null();

            let invalid_ptr = std::ptr::NonNull::dangling();
            let invalid_ptr = invalid_ptr.as_ptr();

            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(null_ptr, ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(invalid_ptr, ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );

            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(ptr, null_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(ptr, invalid_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );

            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(invalid_ptr, null_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(null_ptr, invalid_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
        };
    }

    // link/17.t
    assert_ptr_invalid!(link);
    // rename/17.t
    assert_ptr_invalid!(rename);
    // symlink/13.t
    assert_ptr_invalid!(symlink);
}
