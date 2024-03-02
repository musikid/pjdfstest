//! Tests for writesecurity (called ACL_WRITE_ACL in FreeBSD and
//! ACL_WRITE_SECURITY) in OSX.

use std::{io::ErrorKind, str::FromStr};

use exacl::{AclEntry, AclOption};
use nix::{
    errno::Errno,
    sys::stat::{stat, Mode},
    unistd::chown,
};

use super::prependacl;
use crate::{
    context::{FileType, SerializedTestContext},
    test::FileSystemFeature,
    utils::{chmod, ALLPERMS},
};

crate::test_case! {
    /// ACL_WRITE_ACL does not allow a user to set the SGID bit
    // granular/02.t:L68
    cannot_set_sgid, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn cannot_set_sgid(ctx: &mut SerializedTestContext) {
    let path = ctx
        .new_file(FileType::Regular)
        .mode(0o644)
        .create()
        .unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, None, || {
        let e = chmod(&path, Mode::from_bits_truncate(0o2777));
        assert_eq!(Err(Errno::EPERM), e);
    });
}

crate::test_case! {
    /// ACL_WRITE_ACL does not allow a user to set the SUID bit
    // granular/02.t:L61
    cannot_set_suid, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn cannot_set_suid(ctx: &mut SerializedTestContext) {
    let path = ctx
        .new_file(FileType::Regular)
        .mode(0o644)
        .create()
        .unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, None, || {
        let e = chmod(&path, Mode::from_bits_truncate(0o4777));
        assert_eq!(Err(Errno::EPERM), e);
    });
}

crate::test_case! {
    /// The owner can always read ACLs, even if ACL_READ_ACL is denied
    // granular/02.t:L109
    owner_can_always_write, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn owner_can_always_write(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).mode(0o644).create().unwrap();
    let user = ctx.get_new_user();

    chown(&path, Some(user.uid), Some(user.gid)).unwrap();
    prependacl(&path, &format!("deny::user:{}:writesecurity", user.gid));

    ctx.as_user(user, None, || {
        chmod(&path, Mode::from_bits_truncate(0o777)).unwrap();
    });
}

crate::test_case! {
    /// Writing an ACL via the ACL_SET_WRITE permission does not clear the sgid
    /// bit.
    // granular/02.t:L89
    preserve_sgid, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn preserve_sgid(ctx: &mut SerializedTestContext) {
    let path = ctx
        .new_file(FileType::Regular)
        .mode(0o2755)
        .create()
        .unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, Some(&[user.gid]), || {
        prependacl(&path, &format!("allow::user:{}:write_data", user.uid));
    });
    assert_eq!(0o2755, stat(&path).unwrap().st_mode & ALLPERMS);
}

crate::test_case! {
    /// Writing an ACL via the ACL_SET_WRITE permission does not clear the
    /// sticky bit.
    // granular/02.t:L96
    preserve_sticky, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn preserve_sticky(ctx: &mut SerializedTestContext) {
    let path = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    chmod(&path, Mode::from_bits_truncate(0o1755)).unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, Some(&[user.gid]), || {
        prependacl(&path, &format!("allow::user:{}:write_data", user.uid));
    });
    assert_eq!(0o1755, stat(&path).unwrap().st_mode & ALLPERMS);
}

crate::test_case! {
    /// Writing an ACL via the ACL_SET_WRITE permission does not clear the suid
    /// bit.
    // granular/02.t:L82
    preserve_suid, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn preserve_suid(ctx: &mut SerializedTestContext) {
    let path = ctx
        .new_file(FileType::Regular)
        .mode(0o4755)
        .create()
        .unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, Some(&[user.gid]), || {
        prependacl(&path, &format!("allow::user:{}:write_data", user.uid));
    });
    assert_eq!(0o4755, stat(&path).unwrap().st_mode & ALLPERMS);
}

crate::test_case! {
    /// root can always set ACLs, even if ACL_WRITE_ACL is denied
    // granular/02.t:L126
    root_can_always_set, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn root_can_always_set(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).mode(0o644).create().unwrap();
    let user = ctx.get_new_user();

    chown(&path, Some(user.uid), Some(user.gid)).unwrap();
    prependacl(&path, "deny::everyone::writesecurity");
    prependacl(&path, "deny::everyone::readsecurity");

    chmod(&path, Mode::from_bits_truncate(0o777)).unwrap();
}

crate::test_case! {
    /// ACL_WRITE_ACL does allow a user to set the sticky bit
    // granular/02.t:L75
    set_sticky, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn set_sticky(ctx: &mut SerializedTestContext) {
    let path = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let user = ctx.get_new_user();

    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, None, || {
        chmod(&path, Mode::from_bits_truncate(0o1755)).unwrap();
    });
}

crate::test_case! {
    /// ACL_WRITE_ACL allows a user to write ACLs.
    // granular/02.t:L34
    write_acl, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn write_acl(ctx: &mut SerializedTestContext) {
    let path = ctx
        .new_file(FileType::Regular)
        .mode(0o644)
        .create()
        .unwrap();
    let user = ctx.get_new_user();
    let entry = AclEntry::from_str("allow::everyone::read").unwrap();
    let entries = [entry];

    // by default, non-owners may not write ACLs
    ctx.as_user(user, None, || {
        let e = exacl::setfacl(&[&path][..], &entries, AclOption::empty()).unwrap_err();
        assert_eq!(ErrorKind::PermissionDenied, e.kind());
    });
    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, None, || {
        exacl::setfacl(&[&path][..], &entries, AclOption::empty()).unwrap();
    });
}

crate::test_case! {
    /// ACL_WRITE_ACL allows a user to write a file's mode.
    // granular/02.t:L41
    write_mode, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn write_mode(ctx: &mut SerializedTestContext) {
    let path = ctx
        .new_file(FileType::Regular)
        .mode(0o644)
        .create()
        .unwrap();
    let user = ctx.get_new_user();

    // by default, non-owners may not write ACLs
    ctx.as_user(user, None, || {
        let e = chmod(&path, Mode::from_bits_truncate(0o777)).unwrap_err();
        assert_eq!(Errno::EPERM, e);
    });
    prependacl(&path, &format!("allow::user:{}:writesecurity", user.uid));

    ctx.as_user(user, None, || {
        chmod(&path, Mode::from_bits_truncate(0o777)).unwrap();
    });
}
