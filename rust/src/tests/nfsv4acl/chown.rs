//! Tests for chown (called ACL_WRITE_OWNER on FreeBSD and ACL_CHANGE_OWNER on OSX)
use std::{fs, os::unix::fs::MetadataExt};

use nix::{
    errno::Errno,
    fcntl::OFlag,
    unistd::{chown, Gid, Uid},
};

use super::prependacl;
use crate::{
    context::{FileType, SerializedTestContext},
    test::FileSystemFeature,
    utils::chmod,
    Mode,
};

crate::test_case! {
    /// chown clears setuid and setgid when a non-owner changes gid
    // granular/06.t
    clear_setuid_on_chown_gid, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn clear_setuid_on_chown_gid(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).create().unwrap();
    let user = ctx.get_new_user();
    let group = ctx.get_new_group();

    chmod(&path, Mode::from_bits_truncate(0o6555)).unwrap();
    prependacl(&path, &format!("allow::user:{}:chown", user.gid));

    ctx.as_user(user, Some(&[user.gid, group.gid][..]), || {
        chown(&path, None, Some(group.gid)).unwrap();
    });
    let md = fs::metadata(&path).unwrap();
    assert_eq!(md.mode() & 0o6000, 0);
    assert_eq!(Gid::from(md.gid()), group.gid);
}

crate::test_case! {
    /// chown should not clear setuid and setgid when a non-owner calls chown but changes nothing.
    // granular/06.t
    clear_setuid_on_chown_nothing, serialized, root,
        FileSystemFeature::Nfsv4Acls => [Regular, Dir]
}
fn clear_setuid_on_chown_nothing(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).create().unwrap();
    let user = ctx.get_new_user();

    chmod(&path, Mode::from_bits_truncate(0o6555)).unwrap();
    prependacl(&path, &format!("allow::user:{}:chown", user.gid));

    ctx.as_user(user, None, || {
        chown(&path, None, None).unwrap();
    });
    let md = fs::metadata(&path).unwrap();
    assert_eq!(md.mode() & 0o6000, 0o6000);
}

crate::test_case! {
    /// chown clears setuid and setgid when a non-owner changes uid
    // granular/06.t
    clear_setuid_on_chown_uid, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn clear_setuid_on_chown_uid(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.new_file(ft).create().unwrap();
    let user = ctx.get_new_user();

    chmod(&path, Mode::from_bits_truncate(0o6555)).unwrap();
    prependacl(&path, &format!("allow::user:{}:chown", user.gid));

    ctx.as_user(user, None, || {
        chown(&path, Some(user.uid), None).unwrap();
    });
    let md = fs::metadata(&path).unwrap();
    assert_eq!(md.mode() & 0o6000, 0);
    assert_eq!(Uid::from(md.uid()), user.uid);
}

crate::test_case! {
    /// ACL_WRITE_OWNER allows a user to change a file's GID to his own
    // granular/04.t:L21
    gid, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn gid(ctx: &mut SerializedTestContext) {
    let (path, _file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();
    let user0 = ctx.get_new_user();
    let user1 = ctx.get_new_user();

    // Without any ACL, user0 can't change the gid
    ctx.as_user(user0, None, || {
        assert_eq!(Err(Errno::EPERM), chown(&path, None, Some(user0.gid)));
    });

    prependacl(&path, &format!("allow::user:{}:chown", user0.uid));

    // Even with the ACL, user0 can't change gid to somebody else's
    ctx.as_user(user0, None, || {
        assert_eq!(Err(Errno::EPERM), chown(&path, None, Some(user1.gid)));
    });

    // But he can change it to his own
    ctx.as_user(user0, None, || {
        chown(&path, None, Some(user0.gid)).unwrap();
    });
}

crate::test_case! {
    /// ACL_WRITE_OWNER allows a user to change a file's UID to his own
    // granular/04.t:L33
    uid, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn uid(ctx: &mut SerializedTestContext) {
    let (path, _file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();
    let user0 = ctx.get_new_user();
    let user1 = ctx.get_new_user();

    // Without any ACL, user0 can't change the uid
    ctx.as_user(user0, None, || {
        assert_eq!(Err(Errno::EPERM), chown(&path, Some(user0.uid), None));
    });

    prependacl(&path, &format!("allow::user:{}:chown", user0.uid));

    // Even with the ACL, user0 can't change uid to somebody else's
    ctx.as_user(user0, None, || {
        assert_eq!(Err(Errno::EPERM), chown(&path, Some(user1.uid), None));
    });

    // But he can change it to his own
    ctx.as_user(user0, None, || {
        chown(&path, Some(user0.uid), None).unwrap();
    });
}
