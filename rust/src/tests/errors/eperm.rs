use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    utils::link,
};

use nix::{
    errno::Errno,
    unistd::{chown, unlink},
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
mod flag;

// From https://pubs.opengroup.org/onlinepubs/9699919799/functions/unlink.html
//
// The standard developers reviewed TR 24715-2006 and noted that LSB-conforming implementations
// may return [EISDIR] instead of [EPERM] when unlinking a directory.
// A change to permit this behavior by changing the requirement for [EPERM] to [EPERM] or [EISDIR] was considered,
// but decided against since it would break existing strictly conforming and conforming applications.
// Applications written for portability to both POSIX.1-2017 and the LSB should be prepared to handle either error code.
#[cfg(not(target_os = "linux"))]
crate::test_case! {
    /// unlink may return EPERM if the named file is a directory
    // unlink/08.t
    unlink_dir
}
#[cfg(not(target_os = "linux"))]
fn unlink_dir(ctx: &mut TestContext) {
    let dir = ctx.create(FileType::Dir).unwrap();
    assert!(matches!(unlink(&dir), Ok(_) | Err(Errno::EPERM)));
}

// #[cfg(target_os = "linux")]
// crate::test_case! {
//     /// unlink return EISDIR if the named file is a directory
//     // unlink/08.t
//     unlink_dir
// }
// #[cfg(target_os = "linux")]
// fn unlink_dir(ctx: &mut TestContext) {
//     let dir = ctx.create(FileType::Dir).unwrap();
//     assert!(matches!(unlink(&dir), Err(Errno::EISDIR)));
// }

crate::test_case! {
    /// link returns EPERM if the source file is a directory
    // link/11.t
    link_source_dir, serialized, root
}
fn link_source_dir(ctx: &mut SerializedTestContext) {
    let src = ctx.create(FileType::Dir).unwrap();
    // TODO: Doesn't seem to be an error for SunOS with UFS?
    assert_eq!(link(&src, &ctx.gen_path()), Err(Errno::EPERM));

    let user = ctx.get_new_user();
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
    chown(&src, Some(user.uid), Some(user.gid)).unwrap();

    ctx.as_user(&user, None, || {
        assert_eq!(link(&src, &ctx.gen_path()), Err(Errno::EPERM));
    })
}
