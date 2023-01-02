//! Tests for ACL_DELETE_CHILD
use nix::{errno::Errno, unistd::unlink};

use super::prependacl;
use crate::{
    context::{FileBuilder, FileType, SerializedTestContext},
    test::FileSystemFeature,
    utils::{rename, rmdir},
};

crate::test_case! {
    /// DELETE_CHILD allows for for moving a file out of the target directory,
    /// but not for moving it back.
    // granular/03.t:L115
    // granular/05.t:L124
    allows_rename, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn allows_rename(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let dir1 = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let file = FileBuilder::new(ft, &dir0).mode(0o777).create().unwrap();
    let newpath = dir1.join("new");

    prependacl(&dir0, &format!("allow::user:{}:delete_child", user.uid));

    ctx.as_user(&user, None, move || {
        rename(&file, &newpath).unwrap();
        assert_eq!(Err(Errno::EACCES), rename(&newpath, &file));
    });
}

crate::test_case! {
    /// DELETE_CHILD allows for rmdir, no matter what the permissions on the
    /// parent directory are.
    // granular/05.t:L119
    allows_rmdir, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn allows_rmdir(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let dir1 = FileBuilder::new(FileType::Dir, &dir0)
        .mode(0o755)
        .create()
        .unwrap();

    prependacl(&dir0, &format!("allow::user:{}:delete_child", user.uid));

    ctx.as_user(&user, None, move || {
        rmdir(&dir1).unwrap();
    });
}

crate::test_case! {
    /// DELETE_CHILD allows for unlinking, no matter what the permissions on the
    /// parent directory are.
    // granular/03.t:L110
    allows_unlink, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn allows_unlink(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let file = FileBuilder::new(FileType::Regular, &dir).create().unwrap();

    prependacl(&dir, &format!("allow::user:{}:delete_child", user.uid));

    ctx.as_user(&user, None, move || {
        unlink(&file).unwrap();
    });
}

crate::test_case! {
    /// Denied DELETE_CHILD prohibits unlink, even if the directory is writable.
    /// the directory.
    // granular/03.t:L63
    denied_unlink, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn denied_unlink(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let file = FileBuilder::new(FileType::Regular, &dir).create().unwrap();

    prependacl(&dir, &format!("deny::user:{}:delete_child", user.uid));

    ctx.as_user(&user, None, move || {
        assert_eq!(Errno::EPERM, unlink(&file).unwrap_err());
    });
}

crate::test_case! {
    /// Denied DELETE_CHILD prohibits rmdir, even if the directory is writable.
    /// the directory.
    // granular/05.t:L66
    denied_rmdir, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn denied_rmdir(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let file = FileBuilder::new(FileType::Dir, &dir).create().unwrap();

    prependacl(&dir, &format!("deny::user:{}:delete_child", user.uid));

    ctx.as_user(&user, None, move || {
        assert_eq!(Errno::EPERM, rmdir(&file).unwrap_err());
    });
}

crate::test_case! {
    /// Denied DELETE_CHILD prohibits moving a file out, even if the directory
    /// is writable.  the directory.
    // granular/03.t:L68
    denied_rename, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn denied_rename(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let dir0 = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let dir1 = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let file = FileBuilder::new(ft, &dir0).mode(0o777).create().unwrap();
    let newpath = dir1.join("new");

    prependacl(&dir0, &format!("deny::user:{}:delete_child", user.uid));

    ctx.as_user(&user, None, move || {
        assert_eq!(Err(Errno::EPERM), rename(&file, &newpath));
    });
}
