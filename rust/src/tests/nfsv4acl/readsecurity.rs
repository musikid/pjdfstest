//! Tests for readsecurity (called ACL_READ_ACL in FreeBSD and
//! ACL_READ_SECURITY) in OSX.

use std::io::ErrorKind;

use exacl::{AclOption, getfacl};
use nix::{sys::stat::stat, unistd::chown};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    test::FileSystemFeature,
};
use super::prependacl;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
crate::test_case! {
    /// ACL_READ_ACL allows a user to read ACLs.
    // granular/02.t:L26
    allowed, serialized, root, FileSystemFeature::Nfsv4Acls
}
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn allowed(ctx: &mut SerializedTestContext) {
    let path = ctx.new_file(FileType::Regular).mode(0o644).create().unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("deny::user:{}:readsecurity", user.gid));
    prependacl(&path, &format!("allow::user:{}:readsecurity", user.gid));

    ctx.as_user(&user, None, || {
        getfacl(&path, AclOption::empty()).unwrap();
    });
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
crate::test_case! {
    /// ACL_READ_ACL denied prohibits a user from reading acls
    // granular/02.t:L26
    denied, serialized, root, FileSystemFeature::Nfsv4Acls
}
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn denied(ctx: &mut SerializedTestContext) {
    let path = ctx.new_file(FileType::Regular).mode(0o644).create().unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("deny::user:{}:readsecurity", user.gid));

    ctx.as_user(&user, None, || {
        let e = getfacl(&path, AclOption::empty()).unwrap_err();
        assert_eq!(ErrorKind::PermissionDenied, e.kind());
    });
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
crate::test_case! {
    /// The owner can always read ACLs, even if ACL_READ_ACL is denied
    // granular/02.t:L109
    owner_can_always_read, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn owner_can_always_read(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).mode(0o644).create().unwrap();
    let user = ctx.get_new_user();

    chown(&path, Some(user.uid), Some(user.gid)).unwrap();
    prependacl(&path, &format!("deny::user:{}:readsecurity", user.gid));

    ctx.as_user(&user, None, || {
        getfacl(&path, AclOption::empty()).unwrap();
        stat(&path).unwrap();
    });
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
crate::test_case! {
    /// root can always read ACLs, even if ACL_READ_ACL is denied
    // granular/02.t:L126
    root_can_always_read, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn root_can_always_read(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).mode(0o644).create().unwrap();
    let user = ctx.get_new_user();

    chown(&path, Some(user.uid), Some(user.gid)).unwrap();
    prependacl(&path, "deny::everyone::readsecurity");

    getfacl(&path, AclOption::empty()).unwrap();
    stat(&path).unwrap();
}


